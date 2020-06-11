use std::io::Write;

use super::ManualBufWriter;

use super::OutputRecord;

use super::ProgressStatus;

/// How much bytes may be buffered before flushing
const BYTES_PER_FLUSH: usize = 512 << 10; // 512 KiB

/// Save cursor position and attributes
const VT100_SAVE_CURSOR: &'static [u8] = b"\x1B7";

/// Restore cursor position and attributes
const VT100_RESTORE_CURSOR: &'static [u8] = b"\x1B8";

/// Clear screen from cursor down
const VT100_CLEAR_BELOW: &'static [u8] = b"\x1B[J";

/// Prints `OutputRecord`s, assuming it's called from within a loop.
///
/// Implementations may perform output buffering or progress reporting.
pub trait RecordPrinter {
    /// The main function of `RecordPrinter`. Should be called in a loop.
    ///
    /// `more` hints how many elements are left to be processed after the current one. Hints that
    /// indicate fewer elements than estimated previously will be disregarded.
    ///
    /// It should be called even when there is nothing to print (just use the empty record) — to
    /// provide accurate (and frequent enough) progress reports.
    fn print(&mut self, record: &OutputRecord, more: usize);

    /// Should be called after the loop is finished. It will flush output and clear the progress
    /// report.
    fn finish(&mut self);
}

/// `RecordPrinter` which implements output buffering and progress reporting
pub struct ProgressiveRecordPrinter<O, E>
where
    O: Write,
    E: Write,
{
    stdout: ManualBufWriter<O>,
    stderr: ManualBufWriter<E>,

    status: ProgressStatus,
    last_percent: u32,
}

impl<O, E> ProgressiveRecordPrinter<O, E>
where
    O: Write,
    E: Write,
{
    /// Constructor. `total_hint` is used for the denominator in progress reports.
    pub fn new(stdout: O, stderr: E, total_hint: usize) -> Self {
        Self {
            stdout: ManualBufWriter::new(stdout, 2 * BYTES_PER_FLUSH),
            stderr: ManualBufWriter::new(stderr, BYTES_PER_FLUSH),

            status: ProgressStatus {
                total_no: total_hint,
                processed_no: 0,
            },

            last_percent: 0,
        }
    }
}

impl<O, E> RecordPrinter for ProgressiveRecordPrinter<O, E>
where
    O: Write,
    E: Write,
{
    fn print(&mut self, record: &OutputRecord, more: usize) {
        self.stdout.write_all(record.stdout.as_slice()).unwrap();
        self.stderr.write_all(record.stderr.as_slice()).unwrap();

        self.status.processed();
        self.status.estimate_more(more);

        if self.stdout.len() < BYTES_PER_FLUSH {
            return;
        }

        let cur_percent = (self.status.processed_no * 100 / self.status.total_no) as u32;
        if self.last_percent == cur_percent {
            return;
        }

        self.last_percent = cur_percent;

        // stderr needs to be flushed first, so that the progress bar line is cleared before we
        // print more output
        self.stderr.flush().unwrap();
        self.stdout.flush().unwrap();

        self.stderr.write_all(VT100_SAVE_CURSOR).unwrap();
        write!(
            &mut self.stderr,
            "Files processed: {}/{} ({}%)",
            self.status.processed_no, self.status.total_no, cur_percent
        )
        .unwrap();

        // now we print the progress bar
        self.stderr.flush().unwrap();

        // we leave line clearing codes in the buffer for the next flush when we will want
        // to print more output
        self.stderr.write_all(VT100_RESTORE_CURSOR).unwrap();
        self.stderr.write_all(VT100_CLEAR_BELOW).unwrap();
    }

    fn finish(&mut self) {
        self.stdout.flush().unwrap();
        self.stderr.flush().unwrap();
    }
}

/// `RecordPrinter` which implements output buffering, but not progress reporting
pub struct PlainRecordPrinter<O, E>
where
    O: Write,
    E: Write,
{
    stdout: ManualBufWriter<O>,
    stderr: ManualBufWriter<E>,
}

impl<O, E> PlainRecordPrinter<O, E>
where
    O: Write,
    E: Write,
{
    /// Constructor.
    pub fn new(stdout: O, stderr: E) -> Self {
        Self {
            stdout: ManualBufWriter::new(stdout, 2 * BYTES_PER_FLUSH),
            stderr: ManualBufWriter::new(stderr, BYTES_PER_FLUSH),
        }
    }
}

impl<O, E> RecordPrinter for PlainRecordPrinter<O, E>
where
    O: Write,
    E: Write,
{
    fn print(&mut self, record: &OutputRecord, _: usize) {
        self.stdout.write_all(record.stdout.as_slice()).unwrap();
        self.stderr.write_all(record.stderr.as_slice()).unwrap();

        if BYTES_PER_FLUSH <= self.stdout.len() {
            self.stdout.flush().unwrap();
            self.stderr.flush().unwrap();
        }
    }

    fn finish(&mut self) {
        self.stdout.flush().unwrap();
        self.stderr.flush().unwrap();
    }
}
