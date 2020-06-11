use std::io::ErrorKind;

use std::os::unix::ffi::OsStrExt;

use std::path::Path;
use std::path::PathBuf;

use percent_encoding::{utf8_percent_encode, CONTROLS};

use crate::verdict::Verdict;

use super::LineStatus;
use super::LineStatusColorCodes;

use super::OutputRecord;

use super::RecordPrinter;

/// Returns a human-readable string describing `ek`.
pub fn fmt_error_kind(ek: ErrorKind) -> &'static str {
    match ek {
        ErrorKind::NotFound => "file not found",
        ErrorKind::PermissionDenied => "permission denied",
        ErrorKind::Interrupted => "file reading was interrupted",
        ErrorKind::InvalidData => "invalid data",
        _ => "unexpected error",
    }
}

/// UTF-8 percent-encodes `path`.
pub fn utf8_percent_encode_path(path: &Path) -> String {
    let unicode_path = path.to_string_lossy();
    utf8_percent_encode(&unicode_path, &CONTROLS).to_string()
}

/// Determines how records are printed.
struct OutputSetup {
    color_codes: LineStatusColorCodes,
    line_terminator: &'static [u8],
}

/// Converts `(status, blob)` into the `stdout` (or `stderr`) part of `OutputRecord` according to
/// `OutputSetup`.
///
/// There are 2 usecases for this function:
///
/// 1. `blob` is a path serialized into a sequence of bytes (it's
///     not known at this point which serialization schema was chosen)
/// 2. `blob` is an error message encoded with UTF-8.
///
/// This function doesn't distinguish between these cases and treats `blob` as a — wait for it —
/// blob.
fn wrap_blob_in_record(setup: &OutputSetup, status: LineStatus, blob: &[u8]) -> Vec<u8> {
    let prefix = [status.indicator() as u8, ' ' as u8];

    let components = [
        setup.color_codes.get(status),
        &prefix,
        blob,
        setup.color_codes.reset,
        setup.line_terminator,
    ];

    Vec::from(components.concat())
}

/// Converts `ek` into the `stderr` part of `OutputRecord` according to `OutputSetup`.
fn error_kind_to_stderr_record(setup: &OutputSetup, ek: ErrorKind) -> Vec<u8> {
    wrap_blob_in_record(setup, LineStatus::ErrorDescription, fmt_error_kind(ek).as_bytes())
}

/// Converts `(verdict, blob)` into an `OutputRecord` according to `OutputSetup`.
///
/// `blob` is used to pass paths serialized into byte sequences. Treating them as blobs here
/// facilitates using this function with different path serialization schemas. The function itself
/// doesn't make any assumptions about the contents of `blob`.
fn verdict_and_blob_to_output_record(
    setup: &OutputSetup,
    verdict: Verdict,
    blob: &[u8],
) -> OutputRecord {
    let mut stderr = Vec::new();

    let status = match verdict {
        Verdict::Same => std::unreachable!(),
        Verdict::Deleted => LineStatus::Deleted,
        Verdict::Added => LineStatus::Added,
        Verdict::Modified => LineStatus::Modified,
        Verdict::Error(ek) => {
            stderr = error_kind_to_stderr_record(&setup, ek);
            LineStatus::Error
        }
    };

    let stdout = wrap_blob_in_record(&setup, status, blob);

    OutputRecord { stdout, stderr }
}

/// Converts `(verdict, path)` into an `OutputRecord` according to `OutputSetup`.
/// `path` is utf8-percent-encoded.
fn verdict_and_path_to_percent_output_record(
    setup: &OutputSetup,
    (verdict, path): (Verdict, PathBuf),
) -> OutputRecord {
    if verdict == Verdict::Same {
        OutputRecord::empty()
    } else {
        let percent_path = utf8_percent_encode_path(&path);
        verdict_and_blob_to_output_record(setup, verdict, percent_path.as_bytes())
    }
}

/// Converts `(verdict, path)` into an `OutputRecord` according to `OutputSetup`.
/// `path` is serialiazed into the underlying raw bytes representation.
fn verdict_and_path_to_raw_output_record(
    setup: &OutputSetup,
    (verdict, path): (Verdict, PathBuf),
) -> OutputRecord {
    if verdict == Verdict::Same {
        OutputRecord::empty()
    } else {
        let path_blob = path.as_os_str().as_bytes();
        verdict_and_blob_to_output_record(setup, verdict, path_blob)
    }
}

/// Prints `records` using `record_printer`.
fn print_all_records<I, P>(mut records: I, record_printer: &mut P)
where
    I: Iterator<Item = OutputRecord>,
    P: RecordPrinter,
{
    while let Some(r) = records.next() {
        record_printer.print(&r, records.size_hint().0);
    }

    record_printer.finish();
}

/// Print diff from `verdicts` as specified by `args`
pub fn print_diff<I, P>(
    verdicts: I,
    mut record_printer: P,
    color_codes: LineStatusColorCodes,
    nul_terminated: bool,
) where
    I: Iterator<Item = (Verdict, PathBuf)>,
    P: RecordPrinter,
{
    let line_terminator: &'static [u8] = if nul_terminated { b"\x00" } else { b"\n" };

    let output_setup = OutputSetup {
        color_codes,
        line_terminator,
    };

    let verdict_and_path_to_output_record = |vp| {
        if nul_terminated {
            verdict_and_path_to_raw_output_record(&output_setup, vp)
        } else {
            verdict_and_path_to_percent_output_record(&output_setup, vp)
        }
    };

    let output_records = verdicts.map(verdict_and_path_to_output_record);

    print_all_records(output_records, &mut record_printer);
}
