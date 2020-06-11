/// Single record of `durduff` output
///
/// `durduff` output may be naturally split into records, each consisting of the `stdout` part and
/// the `stderr` part.
///
/// `stdout` is part of the report suitable for parsing (as well as reading by a human unless
/// `--null` option was used).
///
/// `stderr` contains details about the errors indicated in the report with "!". Each "!" line in
/// `stdout` should be paired with a corresponding "^" line in `stderr`.
///
/// `stderr` should be empty for records reporting different statuses than error ("!").
pub struct OutputRecord {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl OutputRecord {
    /// Empty record
    ///
    /// Should be passed to `RecordPrinter::print` whenever both files are the same,
    /// to provide accurate (and frequent enough) progress reports.
    pub fn empty() -> OutputRecord {
        OutputRecord {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}
