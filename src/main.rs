extern crate atty;

#[macro_use]
extern crate cfg_if;

extern crate getopts;

extern crate percent_encoding;

extern crate term_size;

pub mod cli_args;
pub mod error;
pub mod io;
pub mod iter;
pub mod verdict;
pub mod verdictor;

use std::convert::TryFrom;

use std::path::Path;
use std::path::PathBuf;

use crate::verdict::Verdict;

use crate::cli_args::CliArgs;
use crate::cli_args::CliColor;

use crate::io::DiffPrinter;
use crate::io::LineStatusColorCodes;

use crate::iter::cmp_paths;
use crate::iter::OkIter;
use crate::iter::RecDirIter;
use crate::iter::SumIter;
use crate::iter::SumIterSelector;

use crate::verdictor::Verdictor;

/// Get `dirdiff` version (as specified in Cargo.toml).
fn get_version() -> String {
    format!(
        "{}.{}.{}{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    )
}

/// Result of dirdiff execution.
enum ExecResult {
    AllGood,
    SomeErrors,
    FatalError,
}

impl ExecResult {
    /// Convert `ExecResult` to unix exit code.
    fn exit_code(&self) -> i32 {
        match self {
            ExecResult::AllGood => 0,
            ExecResult::SomeErrors => 1,
            ExecResult::FatalError => 2,
        }
    }
}

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();

    let program_name = &raw_args[0];

    let args = match CliArgs::try_from(&raw_args[1..]) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("{}: CLI error: {}", program_name, e);
            std::process::exit(ExecResult::FatalError.exit_code());
        }
    };

    if args.help {
        let brief = format!("Usage: {} [options] OLD NEW", program_name);
        eprint!("{}", args.usage(&brief));
        return;
    }

    if args.version {
        eprintln!("{} {}", program_name, get_version());
        return;
    }

    let exec_result = run_diff(program_name, &args);
    std::process::exit(exec_result.exit_code());
}

/// Estimate the total number of files to process, when comparing directories `lhs` and `rhs`.
/// Useful for progress reporting.
fn calc_total(lhs: &Path, rhs: &Path) -> usize {
    let lhs_iter = RecDirIter::from(lhs.to_path_buf()).filter_map(Result::ok);
    let rhs_iter = RecDirIter::from(rhs.to_path_buf()).filter_map(Result::ok);

    SumIter::new(lhs_iter, rhs_iter, cmp_paths).count()
}

/// Print diff from `verdicts` as specified by `args`.
pub fn print_diff<I>(verdicts: I, args: &CliArgs) -> usize
where
    I: Iterator<Item = (PathBuf, error::Result<Verdict>)>,
{
    let color_codes = match (&args.color, atty::is(atty::Stream::Stdout)) {
        (CliColor::Never, _) | (CliColor::Auto, false) => LineStatusColorCodes::no_color(),
        (CliColor::Always, _) | (CliColor::Auto, true) => LineStatusColorCodes::color(),
    };

    let mut printer = DiffPrinter::new(color_codes, args.percent_paths);

    if !args.progress {
        let mut sink = std::io::sink();
        return printer.print_full(verdicts, args.brief, 0, &mut sink);
    }

    eprint!("calculating totals... ");
    let total_hint = calc_total(&args.old_dir, &args.new_dir);
    eprintln!("done.");

    let stderr = std::io::stderr();
    let mut stderr_guard = stderr.lock();

    printer.print_full(verdicts, args.brief, total_hint, &mut stderr_guard)
}

/// Compare `args.old_dir` and `args.new_dir`.
fn run_diff(program_name: &str, args: &CliArgs) -> ExecResult {
    let lhs_dir_iter = RecDirIter::from(args.old_dir.clone());
    let rhs_dir_iter = RecDirIter::from(args.new_dir.clone());

    let mut lhs_io_err = None;
    let mut rhs_io_err = None;

    let lhs_ok_iter = OkIter::new(lhs_dir_iter, &mut lhs_io_err);
    let rhs_ok_iter = OkIter::new(rhs_dir_iter, &mut rhs_io_err);

    let sum_dir_iter = SumIter::new(lhs_ok_iter, rhs_ok_iter, cmp_paths);

    let mut verdictor = Verdictor::new(&args.old_dir, &args.new_dir, args.block_size);

    let get_verdict = |(selector, path): (SumIterSelector, PathBuf)| {
        let verdict = verdictor.get_verdict(&selector, &path);
        (path, verdict)
    };

    let verdicts = sum_dir_iter.map(get_verdict);

    let err_no = print_diff(verdicts, args);

    if let Some(e) = lhs_io_err.or(rhs_io_err) {
        eprintln!("{}: fatal error: {}", program_name, e);
        ExecResult::FatalError
    } else if 0 < err_no {
        eprintln!("{}: number of errors: {}", program_name, err_no);
        ExecResult::SomeErrors
    } else {
        ExecResult::AllGood
    }
}
