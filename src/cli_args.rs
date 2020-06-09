use std::convert::TryFrom;

use std::fmt;

use std::path::PathBuf;

use std::str::FromStr;

/// Cli option deciding when to print colored output.
#[derive(Clone, Copy)]
pub enum CliColor {
    /// suppress coloring output
    Never,

    /// force coloring output
    Always,

    /// print colored output if and only if stdout is attached to a tty
    Auto,
}

/// Cli option deciding when to report progress.
#[derive(Clone, Copy)]
pub enum CliProgress {
    /// suppress progress reporting
    Never,

    /// force progress reporting
    Always,

    /// report progress if and only if stdout is attached to a tty
    Auto,
}

/// Result of a successful CLI args parsing.
pub struct CliArgs {
    opts: getopts::Options,

    pub help: bool,
    pub version: bool,

    pub brief: bool,

    pub color: CliColor,

    pub progress: CliProgress,

    pub nul_terminated: bool,

    pub block_size: Option<usize>,

    pub old_dir: PathBuf,
    pub new_dir: PathBuf,
}

impl CliArgs {
    pub fn short_usage(&self, program_name: &str) -> String {
        self.opts.short_usage(program_name)
    }

    pub fn usage(&self, brief: &str) -> String {
        self.opts.usage(brief)
    }
}

/// CLI args parsing error.
#[derive(Debug)]
pub enum CliError {
    ArgumentMissing(String),
    UnrecognizedOption(String),
    OptionMissing(String),
    OptionDuplicated(String),
    UnexpectedArgument(String),
    InvalidBlockSize(String),
    InvalidColor(String),
    InvalidProgress(String),
    UnexpectedFreeArgs(String),
    MissingOldDir,
    MissingNewDir,
}

impl FromStr for CliColor {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(CliColor::Never),
            "always" => Ok(CliColor::Always),
            "auto" => Ok(CliColor::Auto),
            s => Err(CliError::InvalidColor(s.to_string())),
        }
    }
}

impl FromStr for CliProgress {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(CliProgress::Never),
            "always" => Ok(CliProgress::Always),
            "auto" => Ok(CliProgress::Auto),
            s => Err(CliError::InvalidProgress(s.to_string())),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::ArgumentMissing(s) => write!(f, "argument missing: {}", s),
            CliError::UnrecognizedOption(s) => write!(f, "unrecognized option: {}", s),
            CliError::OptionMissing(s) => write!(f, "option missing: {}", s),
            CliError::OptionDuplicated(s) => write!(f, "option duplicated: {}", s),
            CliError::UnexpectedArgument(s) => write!(f, "unexpected argument: {}", s),
            CliError::InvalidBlockSize(s) => write!(f, "invalid block size: {}", s),
            CliError::InvalidColor(s) => write!(f, "invalid color: {}", s),
            CliError::InvalidProgress(s) => write!(f, "invalid progress: {}", s),
            CliError::UnexpectedFreeArgs(s) => write!(f, "unexpected free arguments: {}", s),
            CliError::MissingOldDir => write!(f, "missing OLD directory path"),
            CliError::MissingNewDir => write!(f, "missing NEW directory path"),
        }
    }
}

impl From<getopts::Fail> for CliError {
    fn from(f: getopts::Fail) -> Self {
        match f {
            getopts::Fail::ArgumentMissing(s) => CliError::ArgumentMissing(s),
            getopts::Fail::UnrecognizedOption(s) => CliError::UnrecognizedOption(s),
            getopts::Fail::OptionMissing(s) => CliError::OptionMissing(s),
            getopts::Fail::OptionDuplicated(s) => CliError::OptionDuplicated(s),
            getopts::Fail::UnexpectedArgument(s) => CliError::UnexpectedArgument(s),
        }
    }
}

impl TryFrom<&[String]> for CliArgs {
    type Error = CliError;

    /// Parse CLI args, assuming `args` are the CLI args (without the initial program exec
    /// path/name).
    fn try_from(args: &[String]) -> Result<Self, Self::Error> {
        let mut opts = getopts::Options::new();

        opts.optflag("q", "brief", "report only when directories differ");
        opts.optopt(
            "",
            "color",
            "print output in color (<when> may be one of: never, always, auto)",
            "<when>",
        );
        opts.optopt(
            "",
            "progress",
            "print progress bar (<when> may be one of: never, always, auto)",
            "<when>",
        );
        opts.optflag(
            "0",
            "null",
            "print file paths as raw bytes without percent-encoding them and \
             use NUL (null character) instead of LF (new line) to separate \
             lines",
        );
        opts.optopt(
            "b",
            "block-size",
            "read files in blocks of <block-size> bytes",
            "<block-size>",
        );
        opts.optflag("h", "help", "print help information and exit");
        opts.optflag("", "version", "print version information and exit");

        let mut matches = opts.parse(args).map_err(CliError::from)?;

        let help = matches.opt_present("help");
        let version = matches.opt_present("version");

        let (old_dir, new_dir) = if !help && !version {
            if matches.free.is_empty() {
                return Err(CliError::MissingOldDir);
            } else if matches.free.len() == 1 {
                return Err(CliError::MissingNewDir);
            } else if 2 < matches.free.len() {
                return Err(CliError::UnexpectedFreeArgs("".to_string()));
            }

            let new_dir = PathBuf::from(matches.free.swap_remove(1));
            let old_dir = PathBuf::from(matches.free.swap_remove(0));

            (old_dir, new_dir)
        } else {
            (PathBuf::new(), PathBuf::new())
        };

        let color: CliColor = if let Some(c) = matches.opt_str("color") {
            c.parse()?
        } else {
            CliColor::Auto
        };

        let progress: CliProgress = if let Some(p) = matches.opt_str("progress") {
            p.parse()?
        } else {
            CliProgress::Auto
        };

        let block_size = if let Some(b) = matches.opt_str("block-size") {
            match b.parse::<usize>() {
                Ok(0) | Err(_) => return Err(CliError::InvalidBlockSize(b.to_string())),
                Ok(b) => Some(b),
            }
        } else {
            None
        };

        Ok(Self {
            opts,
            help,
            version,
            brief: matches.opt_present("brief"),
            progress,
            color,
            nul_terminated: matches.opt_present("null"),
            block_size,
            old_dir,
            new_dir,
        })
    }
}
