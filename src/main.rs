pub mod cli;
pub mod io;
pub mod iter;
#[macro_use] pub mod osvec;
pub mod verdict;
pub mod verdictor;

use std::convert::TryFrom;

use std::ffi::OsString;

use std::io::Write;

use std::os::unix::io::AsRawFd;

use std::path::Path;

use crate::cli::TtyEnabledOutput;
use crate::cli::parse_cli;

use crate::io::fmt_error_kind;
use crate::io::print_diff;
use crate::io::utf8_percent_encode_path;
use crate::io::LineStatusColorCodes;
use crate::io::PlainRecordPrinter;
use crate::io::ProgressiveRecordPrinter;

use crate::iter::cmp_paths;
use crate::iter::OkIter;
use crate::iter::RecDirIter;
use crate::iter::SumIter;

use crate::verdict::Verdict;

use crate::verdictor::Verdictor;

// Provides the `print_build_info` function.
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

/// Returns `durduff` version from `Cargo.toml`.
pub fn get_version() -> String {
    let core = format!(
        "{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
    );

    match option_env!("CARGO_PKG_VERSION_PRE") {
        Some(pre) => format!("{}-{}", core, pre),
        None => core,
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ErrorStatus {
    NoErrors,
    SomeErrors,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum DiffStatus {
    TreesSame,
    TreesDiff,
}

/// Result of `durduff` execution
enum ExecResult {
    NonFatal(ErrorStatus, DiffStatus),
    Fatal,
}

impl ExecResult {
    /// Convert `ExecResult` to unix exit code.
    fn exit_code(&self) -> i32 {
        match self {
            ExecResult::NonFatal(ErrorStatus::NoErrors, DiffStatus::TreesSame) => 0,
            ExecResult::NonFatal(ErrorStatus::NoErrors, DiffStatus::TreesDiff) => 1,
            ExecResult::NonFatal(ErrorStatus::SomeErrors, DiffStatus::TreesSame) => 2,
            ExecResult::NonFatal(ErrorStatus::SomeErrors, DiffStatus::TreesDiff) => 3,
            ExecResult::Fatal => 4,
        }
    }
}

/// Checks whether `stream` is attached to an interactive terminal.
fn is_tty<S: AsRawFd>(stream: &S) -> bool {
    unsafe { libc::isatty(stream.as_raw_fd()) == 1 }
}

fn main() {
    let exit_code = {
        let raw_args: Vec<OsString> = std::env::args_os().collect();

        let locking_stdout = std::io::stdout();
        let locking_stderr = std::io::stderr();

        let stdout = locking_stdout.lock();
        let stderr = locking_stderr.lock();

        let stdout_is_tty = is_tty(&stdout);
        let stderr_is_tty = is_tty(&stderr);

        run_diff(&raw_args, stdout, stderr, stdout_is_tty, stderr_is_tty)
    };

    std::process::exit(exit_code);
}

/// Estimates the total number of files to process, when comparing directories `lhs` and `rhs`.
///
/// Useful for progress reporting.
fn calc_total(lhs: &Path, rhs: &Path) -> usize {
    let lhs_iter = RecDirIter::try_from(lhs.to_path_buf())
        .unwrap()
        .filter_map(Result::ok);
    let rhs_iter = RecDirIter::try_from(rhs.to_path_buf())
        .unwrap()
        .filter_map(Result::ok);

    SumIter::new(lhs_iter, rhs_iter, cmp_paths).count()
}

/// Testable part of `main`
fn run_diff<O, E>(
    args: &[OsString],
    mut stdout: O,
    mut stderr: E,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
) -> i32
where
    O: Write,
    E: Write,
{
    let cli = parse_cli(args);

    let args = match cli.args {
        Ok(args) => args,
        Err(e) => {
            match e.kind {
                clap::ErrorKind::HelpDisplayed | clap::ErrorKind::VersionDisplayed => {
                    write!(&mut stderr, "{}", e).unwrap();
                },
                _ => {
                    write!(&mut stderr, "{}: {}", cli.bin_name, e).unwrap();
                }
            }
            return ExecResult::Fatal.exit_code();
        }
    };

    let lhs_dir_iter = match RecDirIter::try_from(args.old_dir.clone()) {
        Ok(i) => i,
        Err(_) => {
            writeln!(
                &mut stderr,
                "{}: <old> is not a directory: {}",
                cli.bin_name,
                utf8_percent_encode_path(&args.old_dir)
            )
            .unwrap();
            return ExecResult::Fatal.exit_code();
        }
    };

    let rhs_dir_iter = match RecDirIter::try_from(args.new_dir.clone()) {
        Ok(i) => i,
        Err(_) => {
            writeln!(
                &mut stderr,
                "{}: <new> is not a directory: {}",
                cli.bin_name,
                utf8_percent_encode_path(&args.new_dir)
            )
            .unwrap();
            return ExecResult::Fatal.exit_code();
        }
    };

    let mut lhs_io_err = None;
    let mut rhs_io_err = None;

    let lhs_ok_iter = OkIter::new(lhs_dir_iter, &mut lhs_io_err);
    let rhs_ok_iter = OkIter::new(rhs_dir_iter, &mut rhs_io_err);

    let sum_dir_iter = SumIter::new(lhs_ok_iter, rhs_ok_iter, cmp_paths);

    let mut verdictor = Verdictor::new(&args.old_dir, &args.new_dir, args.block_size);

    let mut error_status = ErrorStatus::NoErrors;
    let mut diff_status = DiffStatus::TreesSame;

    let check_verdict = |(v, _): &(Verdict, _)| match v {
        Verdict::Error(_) => error_status = ErrorStatus::SomeErrors,
        Verdict::Same => (),
        _ => diff_status = DiffStatus::TreesDiff,
    };

    let keep_printing = |(v, _): &(Verdict, _)| {
        if args.brief {
            match v {
                Verdict::Error(_) | Verdict::Same => true,
                _ => false,
            }
        } else {
            true
        }
    };

    let verdicts = sum_dir_iter
        .map(|v| verdictor.get_verdict(v))
        .inspect(check_verdict)
        .take_while(keep_printing);

    let progressive = match args.progress {
        TtyEnabledOutput::Never => false,
        TtyEnabledOutput::Auto => stderr_is_tty,
        TtyEnabledOutput::Always => true,
    };

    let color_codes = match (args.color, stdout_is_tty) {
        (TtyEnabledOutput::Never, _) | (TtyEnabledOutput::Auto, false) => LineStatusColorCodes::no_color(),
        (TtyEnabledOutput::Always, _) | (TtyEnabledOutput::Auto, true) => LineStatusColorCodes::color(),
    };

    if progressive {
        writeln!(&mut stderr, "calculating totals... ").unwrap();
        let total_hint = calc_total(&args.old_dir, &args.new_dir);
        writeln!(&mut stderr, "done.\n").unwrap();

        print_diff(
            verdicts,
            ProgressiveRecordPrinter::new(&mut stdout, &mut stderr, total_hint),
            color_codes.clone(),
            args.nul_terminated,
        )
    } else {
        print_diff(
            verdicts,
            PlainRecordPrinter::new(&mut stdout, &mut stderr),
            color_codes.clone(),
            args.nul_terminated,
        )
    }

    if (diff_status, args.brief) == (DiffStatus::TreesDiff, true) {
        writeln!(&mut stderr, "directory trees differ").unwrap();
    }

    let exec_result = if let Some(e) = lhs_io_err.or(rhs_io_err) {
        stderr.write_all(color_codes.error).unwrap();
        let error_desc = fmt_error_kind(e.kind());
        writeln!(
            &mut stderr,
            "{}: fatal error: {}: {}",
            cli.bin_name, error_desc, e
        )
        .unwrap();
        stderr.write_all(color_codes.reset).unwrap();
        ExecResult::Fatal
    } else {
        if error_status == ErrorStatus::SomeErrors {
            stderr.write_all(color_codes.error).unwrap();
            writeln!(&mut stderr, "{}: nonfatal errors encountered", cli.bin_name).unwrap();
            stderr.write_all(color_codes.reset).unwrap();
        }

        ExecResult::NonFatal(error_status, diff_status)
    };

    exec_result.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::os::unix::ffi::OsStringExt;

    struct Outputs {
        stdout: OsString,
        stderr: OsString,
        exit_code: i32,
    }

    struct ExpectedOutputs {
        stdout: &'static str,
        stderr: &'static str,
        exit_code: i32,
    }

    impl From<&ExpectedOutputs> for Outputs {
        fn from(eo: &ExpectedOutputs) -> Self {
            Self {
                stdout: OsString::from(eo.stdout),
                stderr: OsString::from(eo.stderr),
                exit_code: eo.exit_code,
            }
        }
    }

    fn get_raw_args(file_input: &str, args: &[&str]) -> Vec<OsString> {
        let test_dir_path = ["test-data/", file_input.as_ref()].concat();

        let old = [&test_dir_path, "/old"].concat();
        let new = [&test_dir_path, "/new"].concat();

        let options: &[&str] = args.as_ref();

        [&["nomnom"], options, &[old.as_str(), new.as_str()]]
            .concat()
            .iter()
            .copied()
            .map(OsString::from)
            .collect()
    }

    fn run_diff_and_check_outputs(args: &Vec<OsString>, expected: &ExpectedOutputs) {
        fn run_diff_and_gather_outputs(args: &Vec<OsString>) -> Outputs {
            let mut stdout = Vec::<u8>::new();
            let mut stderr = Vec::<u8>::new();

            let exit_code = run_diff(
                args,
                &mut stdout,
                &mut stderr,
                /* stdout_is_tty: */ false,
                /* stderr_is_tty: */ false,
            );

            Outputs {
                stdout: OsString::from_vec(stdout),
                stderr: OsString::from_vec(stderr),
                exit_code,
            }
        }

        let actual = run_diff_and_gather_outputs(&args);

        let expected_ = Outputs::from(expected);

        assert_eq!(expected_.stdout, actual.stdout);
        assert_eq!(expected_.stderr, actual.stderr);
        assert_eq!(expected_.exit_code, actual.exit_code);
    }

    #[test]
    fn identical() {
        let args: &[&[&str]] = &[
            &[],
            &["--brief"],
            &["--null"],
            &["--block-size", "100"],
            &["--brief", "--null"],
            &["--null", "--block-size", "100"],
            &["--brief", "--block-size", "100"],
            &["--brief", "--null", "--block-size", "100"],
        ];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "",
            exit_code: 0,
        };

        for a in args {
            let raw_args = get_raw_args("identical", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn rudimentary_with_newline() {
        let args: &[&[&str]] = &[
            &[],
            &["--block-size", "100"],
        ];

        let expected = ExpectedOutputs {
            stdout: "+ b\n~ c\n~ foo/a\n",
            stderr: "",
            exit_code: 1,
        };

        for a in args {
            let raw_args = get_raw_args("rudimentary", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn rudimentary_with_null() {
        let args: &[&[&str]] = &[
            &["--null"],
            &["--block-size", "100", "--null"],
        ];

        let expected = ExpectedOutputs {
            stdout: "+ b\x00~ c\x00~ foo/a\x00",
            stderr: "",
            exit_code: 1,
        };

        for a in args {
            let raw_args = get_raw_args("rudimentary", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn problematic_file_names_with_newline() {
        let args: &[&[&str]] = &[
            &[],
            &["--block-size", "100"],
        ];

        let expected = ExpectedOutputs {
            stdout: "+ b\n- hello%0Aworld\n~ foo/a\n",
            stderr: "",
            exit_code: 1,
        };

        for a in args {
            let raw_args = get_raw_args("problematic-file-names", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn problematic_file_names_with_null() {
        let args: &[&[&str]] = &[
            &["--null"],
            &["--block-size", "100", "--null"],
        ];

        let expected = ExpectedOutputs {
            stdout: "+ b\x00- hello\nworld\x00~ foo/a\x00",
            stderr: "",
            exit_code: 1,
        };

        for a in args {
            let raw_args = get_raw_args("problematic-file-names", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn different_symlink_targets_same_contents_with_newline() {
        let args: &[&[&str]] = &[
            &[],
            &["--block-size", "100"],
        ];

        let expected = ExpectedOutputs {
            stdout: "- a\n+ b\n~ foo/symlink\n",
            stderr: "",
            exit_code: 1,
        };

        for a in args {
            let raw_args = get_raw_args("different-symlink-targets-same-contents", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn different_symlink_targets_same_contents_with_null() {
        let args: &[&[&str]] = &[
            &["--null"],
            &["--block-size", "100", "--null"],
        ];

        let expected = ExpectedOutputs {
            stdout: "- a\x00+ b\x00~ foo/symlink\x00",
            stderr: "",
            exit_code: 1,
        };

        for a in args {
            let raw_args = get_raw_args("different-symlink-targets-same-contents", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn brief() {
        let file_input: &[&str] = &[
            "rudimentary",
            "problematic-file-names",
            "different-symlink-targets-same-contents",
        ];

        let args: &[&[&str]] = &[
            &["--brief"],
            &["--brief", "--null"],
            &["--brief", "--block-size", "100"],
            &["--brief", "--null", "--block-size", "100"],
        ];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "directory trees differ\n",
            exit_code: 1,
        };

        for fi in file_input {
            for a in args {
                let raw_args = get_raw_args(fi, a);
                run_diff_and_check_outputs(&raw_args, &expected);
            }
        }
    }

    #[test]
    fn old_is_not_a_dir() {
        let args = osvec!["nomnom", "test-data/rudimentary/old/foo/a", "test-data/rudimentary/old"];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "nomnom: <old> is not a directory: test-data/rudimentary/old/foo/a\n",
            exit_code: 4,
        };

        run_diff_and_check_outputs(&args, &expected);
    }

    #[test]
    fn new_is_not_a_dir() {
        let args = osvec!["nomnom", "test-data/rudimentary/old", "test-data/rudimentary/old/foo/a"];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "nomnom: <new> is not a directory: test-data/rudimentary/old/foo/a\n",
            exit_code: 4,
        };

        run_diff_and_check_outputs(&args, &expected);
    }

    #[test]
    fn negative_block_size() {
        let args: &[&[&str]] = &[
            &["--block-size", "-5"],
        ];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "nomnom: error: Found argument '-5' which wasn't expected, or isn't valid in this context\n\n\
                     USAGE:\n    nomnom <old> <new> --block-size <block-size>\n\n\
                     For more information try --help\n",
            exit_code: 4,
        };

        for a in args {
            let raw_args = get_raw_args("", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn alphabetic_block_size() {
        let args: &[&[&str]] = &[
            &["--block-size", "five"],
        ];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "nomnom: error: Invalid value for '--block-size <block-size>': five\n",
            exit_code: 4,
        };

        for a in args {
            let raw_args = get_raw_args("", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }

    #[test]
    fn zero_block_size() {
        let args: &[&[&str]] = &[
            &["--block-size", "0"],
        ];

        let expected = ExpectedOutputs {
            stdout: "",
            stderr: "nomnom: error: Invalid value for '--block-size <block-size>': 0\n",
            exit_code: 4,
        };

        for a in args {
            let raw_args = get_raw_args("", a);
            run_diff_and_check_outputs(&raw_args, &expected);
        }
    }
}
