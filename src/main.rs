extern crate getopts;

extern crate libc;

extern crate percent_encoding;

#[cfg(test)]
extern crate enum_iterator;

pub mod cli_args;
pub mod io;
pub mod iter;
pub mod verdict;
pub mod verdictor;

use std::convert::TryFrom;

use std::io::Write;

use std::os::unix::io::AsRawFd;

use std::path::Path;

use crate::cli_args::CliArgs;
use crate::cli_args::CliColor;
use crate::cli_args::CliProgress;

use crate::io::fmt_error_kind;
use crate::io::LineStatusColorCodes;
use crate::io::print_diff;
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
fn get_version() -> String {
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
        let raw_args: Vec<String> = std::env::args().collect();

        let locking_stdout = std::io::stdout();
        let locking_stderr = std::io::stderr();

        let stdout = locking_stdout.lock();
        let stderr = locking_stderr.lock();

        let stdout_is_tty = is_tty(&stdout);
        let stderr_is_tty = is_tty(&stderr);

        run_diff(
            &raw_args,
            stdout,
            stderr,
            stdout_is_tty,
            stderr_is_tty,
        )
    };

    std::process::exit(exit_code);
}

/// Estimates the total number of files to process, when comparing directories `lhs` and `rhs`.
///
/// Useful for progress reporting.
fn calc_total(lhs: &Path, rhs: &Path) -> usize {
    let lhs_iter = RecDirIter::from(lhs.to_path_buf()).filter_map(Result::ok);
    let rhs_iter = RecDirIter::from(rhs.to_path_buf()).filter_map(Result::ok);

    let count = SumIter::new(lhs_iter, rhs_iter, cmp_paths).count();

    count
}

/// Testable part of `main`
fn run_diff<O, E>(
    raw_args: &[String],
    mut stdout: O,
    mut stderr: E,
    stdout_is_tty: bool,
    stderr_is_tty: bool,
) -> i32
where
    O: Write,
    E: Write,
{
    let program_name = &raw_args[0];

    let args = match CliArgs::try_from(&raw_args[1..]) {
        Ok(args) => args,
        Err(e) => {
            writeln!(&mut stderr, "{}: CLI error: {}", program_name, e).unwrap();
            return ExecResult::Fatal.exit_code();
        }
    };

    if args.help {
        let brief = format!("Usage: {} [options] OLD NEW", program_name);
        writeln!(&mut stdout, "{}", args.usage(&brief)).unwrap();
        return 0;
    }

    if args.version {
        writeln!(&mut stdout, "durduff {}", get_version()).unwrap();
        writeln!(&mut stdout, "\nBuild information:").unwrap();
        print_build_info();
        return 0;
    }

    let lhs_dir_iter = RecDirIter::from(args.old_dir.clone());
    let rhs_dir_iter = RecDirIter::from(args.new_dir.clone());

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
        CliProgress::Never => false,
        CliProgress::Auto => stderr_is_tty,
        CliProgress::Always => true,
    };

    let color_codes = match (args.color, stdout_is_tty) {
        (CliColor::Never, _) | (CliColor::Auto, false) => LineStatusColorCodes::no_color(),
        (CliColor::Always, _) | (CliColor::Auto, true) => LineStatusColorCodes::color(),
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
        writeln!(&mut stderr, "{}: fatal error: {}: {}", program_name, error_desc, e).unwrap();
        stderr.write_all(color_codes.reset).unwrap();
        ExecResult::Fatal
    } else {
        if error_status == ErrorStatus::SomeErrors {
            stderr.write_all(color_codes.error).unwrap();
            writeln!(&mut stderr, "{}: nonfatal errors encountered", program_name).unwrap();
            stderr.write_all(color_codes.reset).unwrap();
        }

        ExecResult::NonFatal(error_status, diff_status)
    };

    exec_result.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;

    use enum_iterator::IntoEnumIterator;

    struct Outputs {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_code: i32,
    }

    #[derive(Clone, Copy, IntoEnumIterator)]
    enum Args {
        Default,
        NulTerminator,
        CustomBlockSize,
        CustomBlockSizeAndNulTerminator,
        Brief,
        BriefAndNulTerminator,
        CustomBlockSizeAndBrief,
        CustomBlockSizeAndBriefAndNulTerminator,
        NegativeBlockSize,
        AlphabeticBlockSize,
        ZeroBlockSize,
    }

    #[derive(Clone, Copy, IntoEnumIterator)]
    enum FileInput {
        Rudimentary,
        ProblematicFileNames,
        Identical,
        DifferentSymlinkTargetsSameContents,
    }

    fn get_test_dir_name(file_input: FileInput) -> &'static str {
        match file_input {
            FileInput::Rudimentary => "rudimentary",
            FileInput::ProblematicFileNames => "problematic-file-names",
            FileInput::Identical => "identical",
            FileInput::DifferentSymlinkTargetsSameContents => {
                "different-symlink-targets-same-contents"
            }
        }
    }

    fn get_raw_args(file_input: FileInput, args: Args) -> Vec<String> {
        let test_dir_path = ["test-data/", get_test_dir_name(file_input)].concat();

        let old = [&test_dir_path, "/old"].concat();
        let new = [&test_dir_path, "/new"].concat();

        let options: &'static [&'static str] = match args {
            Args::Default => &[],
            Args::NulTerminator => &["--null"],
            Args::CustomBlockSize => &["--block-size", "100"],
            Args::CustomBlockSizeAndNulTerminator => &["--block-size", "100", "--null"],
            Args::Brief => &["--brief"],
            Args::BriefAndNulTerminator => &["--brief", "--null"],
            Args::CustomBlockSizeAndBrief => &["--block-size", "100", "--brief"],
            Args::CustomBlockSizeAndBriefAndNulTerminator => {
                &["--block-size", "100", "--brief", "--null"]
            }
            Args::NegativeBlockSize => &["--block-size", "-5"],
            Args::AlphabeticBlockSize => &["--block-size", "five"],
            Args::ZeroBlockSize => &["--block-size", "0"],
        };

        [&["nomnom"], options, &[old.as_str(), new.as_str()]]
            .concat()
            .iter()
            .copied()
            .map(String::from)
            .collect()
    }

    fn get_expected_outputs(file_input: FileInput, args: Args) -> Outputs {
        match (file_input, args) {
            (FileInput::Rudimentary, Args::Default) => Outputs {
                stdout: b"+ b\n~ c\n~ foo/a\n".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::NulTerminator) => Outputs {
                stdout: b"+ b\x00~ c\x00~ foo/a\x00".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::CustomBlockSize) => Outputs {
                stdout: b"+ b\n~ c\n~ foo/a\n".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::CustomBlockSizeAndNulTerminator) => Outputs {
                stdout: b"+ b\x00~ c\x00~ foo/a\x00".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::Brief) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::BriefAndNulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::CustomBlockSizeAndBrief) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::CustomBlockSizeAndBriefAndNulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::Rudimentary, Args::NegativeBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: -5\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::Rudimentary, Args::AlphabeticBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: five\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::Rudimentary, Args::ZeroBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: 0\n".to_vec(),
                exit_code: 4,
            },

            (FileInput::ProblematicFileNames, Args::Default) => Outputs {
                stdout: b"+ b\n- hello%0Aworld\n~ foo/a\n".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::NulTerminator) => Outputs {
                stdout: b"+ b\x00- hello\nworld\x00~ foo/a\x00".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::CustomBlockSize) => Outputs {
                stdout: b"+ b\n- hello%0Aworld\n~ foo/a\n".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::CustomBlockSizeAndNulTerminator) => Outputs {
                stdout: b"+ b\x00- hello\nworld\x00~ foo/a\x00".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::Brief) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::BriefAndNulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::CustomBlockSizeAndBrief) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::ProblematicFileNames, Args::CustomBlockSizeAndBriefAndNulTerminator) => {
                Outputs {
                    stdout: Vec::new(),
                    stderr: b"directory trees differ\n".to_vec(),
                    exit_code: 1,
                }
            }
            (FileInput::ProblematicFileNames, Args::NegativeBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: -5\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::ProblematicFileNames, Args::AlphabeticBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: five\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::ProblematicFileNames, Args::ZeroBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: 0\n".to_vec(),
                exit_code: 4,
            },

            (FileInput::Identical, Args::Default) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::NulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::CustomBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::CustomBlockSizeAndNulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::Brief) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::BriefAndNulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::CustomBlockSizeAndBrief) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::CustomBlockSizeAndBriefAndNulTerminator) => Outputs {
                stdout: Vec::new(),
                stderr: Vec::new(),
                exit_code: 0,
            },
            (FileInput::Identical, Args::NegativeBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: -5\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::Identical, Args::AlphabeticBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: five\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::Identical, Args::ZeroBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: 0\n".to_vec(),
                exit_code: 4,
            },

            (FileInput::DifferentSymlinkTargetsSameContents, Args::Default) => Outputs {
                stdout: b"- a\n+ b\n~ foo/symlink\n".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::DifferentSymlinkTargetsSameContents, Args::NulTerminator) => Outputs {
                stdout: b"- a\x00+ b\x00~ foo/symlink\x00".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::DifferentSymlinkTargetsSameContents, Args::CustomBlockSize) => Outputs {
                stdout: b"- a\n+ b\n~ foo/symlink\n".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (
                FileInput::DifferentSymlinkTargetsSameContents,
                Args::CustomBlockSizeAndNulTerminator,
            ) => Outputs {
                stdout: b"- a\x00+ b\x00~ foo/symlink\x00".to_vec(),
                stderr: Vec::new(),
                exit_code: 1,
            },
            (FileInput::DifferentSymlinkTargetsSameContents, Args::Brief) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::DifferentSymlinkTargetsSameContents, Args::BriefAndNulTerminator) => {
                Outputs {
                    stdout: Vec::new(),
                    stderr: b"directory trees differ\n".to_vec(),
                    exit_code: 1,
                }
            }
            (FileInput::DifferentSymlinkTargetsSameContents, Args::CustomBlockSizeAndBrief) => {
                Outputs {
                    stdout: Vec::new(),
                    stderr: b"directory trees differ\n".to_vec(),
                    exit_code: 1,
                }
            }
            (
                FileInput::DifferentSymlinkTargetsSameContents,
                Args::CustomBlockSizeAndBriefAndNulTerminator,
            ) => Outputs {
                stdout: Vec::new(),
                stderr: b"directory trees differ\n".to_vec(),
                exit_code: 1,
            },
            (FileInput::DifferentSymlinkTargetsSameContents, Args::NegativeBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: -5\n".to_vec(),
                exit_code: 4,
            },
            (FileInput::DifferentSymlinkTargetsSameContents, Args::AlphabeticBlockSize) => {
                Outputs {
                    stdout: Vec::new(),
                    stderr: b"nomnom: CLI error: invalid block size: five\n".to_vec(),
                    exit_code: 4,
                }
            }
            (FileInput::DifferentSymlinkTargetsSameContents, Args::ZeroBlockSize) => Outputs {
                stdout: Vec::new(),
                stderr: b"nomnom: CLI error: invalid block size: 0\n".to_vec(),
                exit_code: 4,
            },
        }
    }

    fn run_diff_and_gather_outputs(raw_args: &Vec<String>) -> Outputs {
        let mut stdout = Vec::<u8>::new();
        let mut stderr = Vec::<u8>::new();

        let exit_code = run_diff(
            raw_args,
            &mut stdout,
            &mut stderr,
            /* stdout_is_tty: */ false,
            /* stderr_is_tty: */ false,
        );

        Outputs {
            stdout,
            stderr,
            exit_code,
        }
    }

    #[test]
    fn all_func_tests() {
        for fi in FileInput::into_enum_iter() {
            for a in Args::into_enum_iter() {
                let raw_args = get_raw_args(fi, a);

                let expected = get_expected_outputs(fi, a);
                let actual = run_diff_and_gather_outputs(&raw_args);

                let utf8_expected_stdout = String::from_utf8_lossy(&expected.stdout);
                let utf8_expected_stderr = String::from_utf8_lossy(&expected.stderr);

                let utf8_actual_stdout = String::from_utf8_lossy(&actual.stdout);
                let utf8_actual_stderr = String::from_utf8_lossy(&actual.stderr);

                assert_eq!(
                    expected.stdout, actual.stdout,
                    "stdout divergence for args: {:?}. utf-8 expected: {:?}. utf-8 actual: {:?}.",
                    &raw_args, utf8_expected_stdout, utf8_actual_stdout
                );
                assert_eq!(
                    expected.stderr, actual.stderr,
                    "stderr divergence for args: {:?}. utf-8 expected: {:?}. utf-8 actual: {:?}.",
                    &raw_args, utf8_expected_stderr, utf8_actual_stderr
                );
                assert_eq!(
                    expected.exit_code, actual.exit_code,
                    "exit code divergence for args: {:?}",
                    &raw_args
                );
            }
        }
    }
}
