use std::ffi::OsStr;
use std::ffi::OsString;

use std::path::PathBuf;

use crate::get_version;

use clap::App;
use clap::Arg;

/// Cli option deciding when to print tty-enabled (e.g. colored) output
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TtyEnabledOutput {
    /// suppress tty-enabled output
    Never,

    /// force tty-enabled output
    Always,

    /// print tty-enabled output if and only if stdout/stderr is attached to a tty
    Auto,
}

/// Result of successfully parsing CLI args
#[derive(Debug, Eq, PartialEq)]
pub struct CliArgs {
    pub brief: bool,
    pub nul_terminated: bool,

    pub color: TtyEnabledOutput,
    pub progress: TtyEnabledOutput,

    pub block_size: Option<usize>,

    pub old_dir: PathBuf,
    pub new_dir: PathBuf,
}

pub struct Cli {
    pub bin_name: String,
    pub args: Result<CliArgs, clap::Error>,
}

/// Parse CLI args, assuming `args` are the CLI args (with the initial program exec path/name).
pub fn parse_cli(args: &[OsString]) -> Cli {
    fn is_valid_block_size(s: String) -> Result<(), String> {
        match s.parse::<usize>() {
            Ok(0) | Err(_) => Err(s),
            Ok(_) => Ok(()),
        }
    }

    let version = get_version();

    let pkg_name = env!("CARGO_PKG_NAME");

    let bin_name = match args.get(0) {
        Some(n) => n.to_str().unwrap_or(pkg_name),
        None => pkg_name,
    };

    let app = App::new(pkg_name)
        .bin_name(bin_name)
        .version(version.as_str())
        .about("Compares directories file by file")
        .arg(Arg::with_name("brief")
            .short("q")
            .long("brief")
            .help("Report only when directories differ")
            .display_order(1))
        .arg(Arg::with_name("null")
            .short("0")
            .long("null")
            .help("Print raw NUL-separated paths")
            .display_order(2))
        .arg(Arg::with_name("color")
            .long("color")
            .value_name("when")
            .help("Print output in color")
            .takes_value(true)
            .possible_values(&["never", "always", "auto"])
            .default_value("auto")
            .display_order(3))
        .arg(Arg::with_name("progress")
            .long("progress")
            .value_name("when")
            .help("Print progress reports")
            .takes_value(true)
            .possible_values(&["never", "always", "auto"])
            .default_value("auto")
            .display_order(4))
        .arg(Arg::with_name("block-size")
            .short("b")
            .long("block-size")
            .value_name("block-size")
            .help("Read files in blocks of <block-size> bytes")
            .takes_value(true)
            .validator(is_valid_block_size)
            .display_order(5))
        .help_message("Print help information and exit")
        .version_message("Print version information and exit")
        .arg(Arg::with_name("old")
            .value_name("old")
            .required_unless_one(&["help", "version"])
            .hidden(true)
            .index(1))
        .arg(Arg::with_name("new")
            .value_name("new")
            .required_unless_one(&["help", "version"])
            .hidden(true)
            .index(2));

    fn parse_after_bin_name(app: clap::App, args: &[OsString]) -> Result<CliArgs, clap::Error> {
        let matches = app.get_matches_from_safe(args)?;

        fn parse_tty_enabled(s: &str) -> TtyEnabledOutput {
            match s {
                "always" => TtyEnabledOutput::Always,
                "auto" => TtyEnabledOutput::Auto,
                "never" => TtyEnabledOutput::Never,
                &_ => unreachable!(),
            }
        }

        let color = parse_tty_enabled(matches.value_of_lossy("color").unwrap().as_ref());
        let progress = parse_tty_enabled(matches.value_of_lossy("progress").unwrap().as_ref());

        fn get_path(o: Option<&OsStr>) -> PathBuf {
            PathBuf::from(o.unwrap_or(OsStr::new("")).to_owned())
        }

        let block_size = matches
            .value_of("block-size")
            .map(|b| b.parse::<usize>().unwrap());

        Ok(CliArgs {
            brief: matches.is_present("brief"),
            nul_terminated: matches.is_present("null"),

            color,
            progress,

            block_size,

            old_dir: get_path(matches.value_of_os("old")),
            new_dir: get_path(matches.value_of_os("new")),
        })
    }

    Cli {
        bin_name: bin_name.to_owned(),
        args: parse_after_bin_name(app, args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use crate::osvec;

    fn parse_utf8_cli(args: &[&str]) -> Cli {
        let os_args: Vec<OsString> = args.iter().copied().map(OsString::from).collect();

        parse_cli(&os_args)
    }

    // binary name (args[0]) different from pkg name (from Cargo.toml)
    #[test]
    fn changed_bin_name() {
        for bin_name in ["first", "last"].iter().copied() {
            let args = [bin_name, "<old>", "<new>"];

            let cli = parse_utf8_cli(&args);

            assert_eq!(cli.bin_name, bin_name);
        }
    }

    #[test]
    fn negative_block_size() {
        let args = ["nomnom", "--block-size", "-5", "<old>", "<new>"];

        let err = parse_utf8_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::UnknownArgument);
    }

    #[test]
    fn alphabetic_block_size() {
        let args = ["nomnom", "--block-size", "five", "<old>", "<new>"];

        let err = parse_utf8_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::ValueValidation);
    }

    #[test]
    fn zero_block_size() {
        let args = ["nomnom", "--block-size", "0", "<old>", "<new>"];

        let err = parse_utf8_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::ValueValidation);
    }

    #[test]
    fn invalid_progress() {
        let args = ["nomnom", "--progress", "sdfs", "<old>", "<new>"];

        let err = parse_utf8_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::InvalidValue);
    }

    #[test]
    fn invalid_color() {
        let args = ["nomnom", "--color", "sdfs", "<old>", "<new>"];

        let err = parse_utf8_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::InvalidValue);
    }

    const NON_UTF8_BYTE_SEQ: &[u8] = &[0xf1, 0x52, 0x88, 0x39, 0x3a, 0xf6, 0x11];

    #[test]
    fn non_utf8_block_size() {
        let args = [
            OsString::from("nomnom"),
            OsString::from("--block-size"),
            OsString::from_vec(NON_UTF8_BYTE_SEQ.to_vec()),
            OsString::from("<old>"),
            OsString::from("<new>"),
        ];

        let err = parse_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::ValueValidation);
    }

    #[test]
    fn non_utf8_progress() {
        let args = [
            OsString::from("nomnom"),
            OsString::from("--progress"),
            OsString::from_vec(NON_UTF8_BYTE_SEQ.to_vec()),
            OsString::from("<old>"),
            OsString::from("<new>"),
        ];

        let err = parse_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::InvalidValue);
    }

    #[test]
    fn non_utf8_color() {
        let args = [
            OsString::from("nomnom"),
            OsString::from("--color"),
            OsString::from_vec(NON_UTF8_BYTE_SEQ.to_vec()),
            OsString::from("<old>"),
            OsString::from("<new>"),
        ];

        let err = parse_cli(&args).args.unwrap_err();

        assert_eq!(err.kind, clap::ErrorKind::InvalidValue);
    }

    /// Returns arguments which should be parsed into `args` (in all possible permutations).
    fn cli_args_to_raw(args: &CliArgs, permute: bool) -> Vec<Vec<OsString>> {
        use permutohedron::Heap;

        let color_variants = match args.color {
            TtyEnabledOutput::Never => vec![osvec!["--color", "never"]],
            TtyEnabledOutput::Always => vec![osvec!["--color", "always"]],
            TtyEnabledOutput::Auto => vec![osvec!["--color", "auto"], osvec![]],
        };

        let progress_variants = match args.progress {
            TtyEnabledOutput::Never => vec![osvec!["--progress", "never"]],
            TtyEnabledOutput::Always => vec![osvec!["--progress", "always"]],
            TtyEnabledOutput::Auto => vec![osvec!["--progress", "auto"], osvec![]],
        };

        let brief_variants = if args.brief {
            vec![osvec!["-q"], osvec!["--brief"]]
        } else {
            vec![vec![]]
        };

        let null_variants = if args.nul_terminated {
            vec![osvec!["-0"], osvec!["--null"]]
        } else {
            vec![vec![]]
        };

        let bs_variants = if let Some(b) = args.block_size {
            let bs = format!("{}", b);
            vec![osvec!["-b", bs.clone()], osvec!["--block-size", bs]]
        } else {
            vec![vec![]]
        };

        let dirnames = osvec![args.old_dir.clone(), args.new_dir.clone()];

        let mut raw_args = Vec::<Vec<OsString>>::new();

        for cv in color_variants {
            for pv in progress_variants.iter() {
                for bv in brief_variants.iter() {
                    for nv in null_variants.iter() {
                        for bsv in bs_variants.iter() {
                            let parts: &[&[OsString]] = &[&cv, &pv, &bv, &nv, &bsv, &dirnames];

                            let mut non_empty_parts: Vec<&[OsString]> =
                                parts.iter().copied().filter(|p| !p.is_empty()).collect();

                            if permute {
                                let all_perms = Heap::new(&mut non_empty_parts).map(|v| v.concat());
                                raw_args.extend(all_perms);
                            } else {
                                raw_args.push(non_empty_parts.concat());
                            }
                        }
                    }
                }
            }
        }

        raw_args
    }

    fn parsing_full_circle(source_args: &CliArgs, permute: bool) {
        let mut raw_args_perms = cli_args_to_raw(source_args, permute);

        for mut raw_args in raw_args_perms.drain(..) {
            let mut full_raw_args = vec![OsString::from("nomnom")];
            full_raw_args.append(&mut raw_args);

            let parsed_cli = parse_cli(&full_raw_args);

            assert_eq!(parsed_cli.bin_name, "nomnom", "args: {:?}", full_raw_args);

            assert!(parsed_cli.args.is_ok(), "args: {:?}", full_raw_args);

            assert_eq!(
                parsed_cli.args.unwrap(),
                *source_args,
                "args: {:?}",
                full_raw_args
            );
        }
    }

    #[test]
    fn full_circle_permuted_1() {
        let source_args = CliArgs {
            brief: true,
            nul_terminated: true,

            color: TtyEnabledOutput::Always,
            progress: TtyEnabledOutput::Never,

            block_size: Some(400),

            old_dir: PathBuf::from("happy"),
            new_dir: PathBuf::from("panda"),
        };

        parsing_full_circle(&source_args, false);
    }

    #[test]
    fn full_circle_permuted_2() {
        let source_args = CliArgs {
            brief: false,
            nul_terminated: true,

            color: TtyEnabledOutput::Never,
            progress: TtyEnabledOutput::Auto,

            block_size: Some(600),

            old_dir: PathBuf::from("inverted"),
            new_dir: PathBuf::from("panda"),
        };

        parsing_full_circle(&source_args, false);
    }

    #[test]
    fn full_circle_1() {
        let source_args = CliArgs {
            brief: true,
            nul_terminated: false,

            color: TtyEnabledOutput::Auto,
            progress: TtyEnabledOutput::Never,

            block_size: Some(1),

            old_dir: PathBuf::from("/foo/bar.txt"),
            new_dir: PathBuf::from("baz"),
        };

        parsing_full_circle(&source_args, false);
    }

    #[test]
    fn full_circle_2() {
        let source_args = CliArgs {
            brief: false,
            nul_terminated: true,

            color: TtyEnabledOutput::Always,
            progress: TtyEnabledOutput::Auto,

            block_size: None,

            old_dir: PathBuf::from("c"),
            new_dir: PathBuf::from(OsString::from_vec(NON_UTF8_BYTE_SEQ.to_vec())),
        };

        parsing_full_circle(&source_args, false);
    }

    #[test]
    fn full_circle_3() {
        let source_args = CliArgs {
            brief: true,
            nul_terminated: false,

            color: TtyEnabledOutput::Never,
            progress: TtyEnabledOutput::Always,

            block_size: Some(512 << 10),

            old_dir: PathBuf::from("foo/bar.txt"),
            new_dir: PathBuf::from("foo/bar"),
        };

        parsing_full_circle(&source_args, false);
    }

    #[test]
    fn full_circle_4() {
        let source_args = CliArgs {
            brief: false,
            nul_terminated: false,

            color: TtyEnabledOutput::Never,
            progress: TtyEnabledOutput::Always,

            block_size: Some(usize::MAX),

            old_dir: PathBuf::from("a/b"),
            new_dir: PathBuf::from("c"),
        };

        parsing_full_circle(&source_args, false);
    }

    #[test]
    fn full_circle_5() {
        let source_args = CliArgs {
            brief: false,
            nul_terminated: false,

            color: TtyEnabledOutput::Never,
            progress: TtyEnabledOutput::Always,

            block_size: Some(usize::MAX),

            old_dir: PathBuf::from("a/b"),
            new_dir: PathBuf::from(""),
        };

        parsing_full_circle(&source_args, false);
    }
}
