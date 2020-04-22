use std::io::ErrorKind;
use std::io::BufWriter;
use std::io::IoSlice;
use std::io::Write;

use std::os::unix::ffi::OsStrExt;

use std::path::Path;
use std::path::PathBuf;

use percent_encoding::{utf8_percent_encode, CONTROLS};

use crate::verdict::Verdict;

use super::FpsBufWriter;
use super::LineStatus;
use super::LineStatusColorCodes;

/// `DiffPrinter` is responsible for putting the diff in a textual format onto stdout.
pub struct DiffPrinter {
    color_codes: LineStatusColorCodes,

    nul_terminated: bool,

    err_no: usize,
}

fn get_tty_width() -> usize {
    match term_size::dimensions() {
        Some((w, _)) => w as usize,
        None => 80,
    }
}


// let out = std::io::stdout();
// let buf_out: Box<dyn Write> = if atty::is(atty::Stream::Stdout) {
//     Box::from(out.lock())
// } else {
//     Box::from(BufWriter::new(out.lock()))
// };

impl DiffPrinter {
    fn print_line<W: Write>(
        &mut self,
        stdout: &mut O,
        stderr: &mut E,
        progress: &mut P,
        status: LineStatus,
        path: &Path
    ) {
        let color_code = self.color_codes.get(status);

        if self.nul_terminated {
            // nul-termination implies no color codes. they would only mess up parsing by other
            // programs

            let raw_bytes = path.as_os_str().as_bytes();

            let indicator_bytes = [status.indicator() as u8];

            let out_vec = [
                IoSlice::new(&indicator_bytes),
                IoSlice::new(b" "),
                IoSlice::new(raw_bytes),
                IoSlice::new(&[0]),
            ];

            out.write_vectored(&out_vec).unwrap();
        } else {
            let unicode_path = path.to_string_lossy();
            let percent_path = utf8_percent_encode(&unicode_path, &CONTROLS);

            writeln!(
                out,
                "{}{} {}{}",
                color_code,
                status.indicator(),
                percent_path,
                self.color_codes.reset
            )
            .unwrap();
        }
    }

    fn print_verdict<W: Write>(
        &mut self,
        stdout: &mut O,
        stderr: &mut E,
        progress: &mut P,
        path: &Path,
        verdict: Verdict
    ) {
        let (status, error_kind) = match verdict {
            Verdict::Error(ek) => (LineStatus::Error, Some(ek)),
            Verdict::Deleted => (LineStatus::Deleted, None),
            Verdict::Added => (LineStatus::Added, None),
            Verdict::Modified => (LineStatus::Modified, None),
            Verdict::Same => return,
        };

        self.print_line(out, status, path);

        if let Some(ek) = error_kind {
            let err_str = match ek {
                ErrorKind::NotFound => "file not found",
                ErrorKind::PermissionDenied => "permission denied",
                ErrorKind::Interrupted => "file reading was interrupted",
                ErrorKind::InvalidData => "invalid data",
                _ => "unexpected error",
            };
            ep
            // print error message
        }
    }

    pub fn print_full<I, O, E>(
        &mut self,
        mut verdicts: I,
        brief: bool,
        min_hint: usize,
        stdout: &mut O,
        stderr: &mut E,
        progress: &mut P,
    ) -> usize
    where
        I: Iterator<Item = (Verdict, PathBuf)>,
        O: Write,
        E: Write,
    {
        let mut progress = ProgressData {
            total_no: min_hint,
            processed_no: 0,
        };

        if brief {
            while let Some((v, p)) = verdicts.next() {
                progress.processed();
                progress.estimate_more(verdicts.size_hint().0);

                match v {
                    Verdict::Error(_) => (),
                    Verdict::Same => continue,
                    _ => {
                        println!("directory trees differ");
                        break;
                    },
                }

                self.print_verdict(&mut fps_writer, &p, v)
            }
        } else {
            while let Some((v, p)) = verdicts.next() {
                progress.processed();
                progress.estimate_more(verdicts.size_hint().0);

                self.print_verdict(&mut fps_writer, &p, v);
            }
        }

        writeln!(progress_out).unwrap();

        self.err_no
    }

    /// Creates a new `DiffPrinter` which will use `color_codes` to write colored output.
    /// `percent_paths` determines if the paths will be percent-encoded.
    pub fn new(color_codes: LineStatusColorCodes, nul_terminated: bool) -> Self {
        Self {
            color_codes,
            nul_terminated,
            err_no: 0,
        }
    }
}
