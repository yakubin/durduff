use std::borrow::Cow;

use std::io;
use std::io::BufWriter;
use std::io::IoSlice;
use std::io::Write;

use std::os::unix::ffi::OsStrExt;

use std::path::Path;
use std::path::PathBuf;

use percent_encoding::{utf8_percent_encode, CONTROLS};

use crate::error::*;

use crate::io::PercentPath;

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

impl DiffPrinter {
    fn print_line<W: Write>(&mut self, out: &mut W, status: LineStatus, path: &Path) {
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

    fn print_verdict<W: Write>(&mut self, out: &mut W, path: &Path, verdict: &Verdict) {
        let status = match verdict {
            Verdict::Deleted => LineStatus::Deleted,
            Verdict::Added => LineStatus::Added,
            Verdict::Modified => LineStatus::Modified,
            Verdict::Same => return,
        };

        self.print_line(out, status, path);
    }

    fn print_error<W: Write>(&self, out: &mut W, path: &Path, error: &Error) {
        let (prefix, io_error) = match error {
            Error::Lhs(e) => (Path::new("OLD"), e),
            Error::Rhs(e) => (Path::new("NEW"), e),
        };

        let full_path = prefix.join(path);

        if self.nul_terminated {
            // nul-termination implies no color codes. they would only mess up parsing by other
            // programs

            let raw_bytes = full_path.as_os_str().as_bytes();

            let out_vec = [
                IoSlice::new(&[LineStatus::Error as u8]),
                IoSlice::new(b" "),
                IoSlice::new(&raw_bytes),
                IoSlice::new(&[0]),
            ];

            out.write_vectored(&out_vec).unwrap();
        } else {
            let color_code = self.color_codes.get(LineStatus::Error);

            let err_str: Cow<str> = match io_error.kind() {
                io::ErrorKind::NotFound => Cow::from("file not found"),
                io::ErrorKind::PermissionDenied => Cow::from("permission denied"),
                io::ErrorKind::Interrupted => Cow::from("file reading was interrupted"),
                io::ErrorKind::InvalidData => Cow::from(format!("invalid data: {}", io_error)),
                _ => Cow::from(format!("unexpected error: {:?}", io_error)),
            };

            writeln!(
                out,
                "{}{} {}: {}{}",
                color_code,
                LineStatus::Error.indicator(),
                err_str,
                PercentPath::from(&full_path),
                self.color_codes.reset
            )
            .unwrap();
        }
    }

    pub fn print<W: Write>(&mut self, out: &mut W, path: &Path, result: &Result<Verdict>) {
        match result {
            Ok(v) => self.print_verdict(out, path, v),
            Err(e) => {
                self.err_no += 1;
                self.print_error(out, path, e);
            }
        }
    }

    pub fn print_full<I>(
        &mut self,
        mut verdicts: I,
        brief: bool,
        min_hint: usize,
        progress_out: &mut dyn Write,
    ) -> usize
    where
        I: Iterator<Item = (PathBuf, Result<Verdict>)>,
    {
        let out = std::io::stdout();
        let buf_out: Box<dyn Write> = if atty::is(atty::Stream::Stdout) {
            Box::from(out.lock())
        } else {
            Box::from(BufWriter::new(out.lock()))
        };

        let mut fps_writer = FpsBufWriter::new(buf_out, progress_out, get_tty_width);

        fps_writer.estimate_more(min_hint);
        fps_writer.estimate_more(verdicts.size_hint().0);

        fps_writer.flush().unwrap();

        let mut dirs_differ = false;

        while let Some((p, v)) = verdicts.next() {
            fps_writer.processed();
            fps_writer.estimate_more(verdicts.size_hint().0);

            if brief {
                match &v {
                    Err(_) => self.print(&mut fps_writer, &p, &v),
                    Ok(Verdict::Same) => (),
                    Ok(_) => dirs_differ = true,
                }
            } else {
                self.print(&mut fps_writer, &p, &v);
            }

            fps_writer.try_flush().unwrap();

            if brief && dirs_differ {
                break;
            }
        }

        fps_writer.flush().unwrap();

        writeln!(progress_out).unwrap();

        if brief && dirs_differ {
            println!("directory trees differ");
        }

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
