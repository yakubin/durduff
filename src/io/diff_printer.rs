use std::borrow::Cow;

use std::io;
use std::io::BufWriter;
use std::io::Write;

use std::path::Path;
use std::path::PathBuf;

use crate::io::PercentPath;

use crate::error::*;
use crate::verdict::Verdict;

use super::FpsBufWriter;
use super::LineStatus;
use super::LineStatusColorCodes;

/// `DiffPrinter` is responsible for putting the diff in a textual format onto stdout.
pub struct DiffPrinter {
    color_codes: LineStatusColorCodes,

    percent_paths: bool,

    err_no: usize,
}

fn get_tty_width() -> usize {
    match term_size::dimensions() {
        Some((w, _)) => w as usize,
        None => 80,
    }
}

impl DiffPrinter {
    fn print_line<W: Write>(&mut self, out: &mut W, status: &LineStatus, path: &Path) {
        let color_code = self.color_codes.get(status);
        let indicator = status.indicator();

        if !self.percent_paths {
            let unicode_path = path.to_string_lossy();

            if !unicode_path.contains('\n') {
                writeln!(
                    out,
                    "{}{} {}{}",
                    color_code, indicator, unicode_path, self.color_codes.reset
                )
                .unwrap();

                return;
            }

            writeln!(
                out,
                "{}{} path contains the LINE FEED character. falling back to percent-encoding.{}",
                self.color_codes.get(&LineStatus::Error),
                LineStatus::Error.indicator(),
                self.color_codes.reset
            )
            .unwrap();

            self.err_no += 1;
        }

        let percent_path = PercentPath::from(path);

        writeln!(
            out,
            "{}{} {}{}",
            color_code, indicator, percent_path, self.color_codes.reset
        )
        .unwrap();
    }

    fn print_verdict<W: Write>(&mut self, out: &mut W, path: &Path, verdict: &Verdict) {
        let status = match verdict {
            Verdict::Deleted => LineStatus::Deleted,
            Verdict::Added => LineStatus::Added,
            Verdict::Modified => LineStatus::Modified,
            Verdict::Same => return,
        };

        self.print_line(out, &status, path);
    }

    fn print_error<W: Write>(&self, out: &mut W, path: &Path, error: &Error) {
        let color_code = self.color_codes.get(&LineStatus::Error);

        let (prefix, io_error) = match error {
            Error::Lhs(e) => (Path::new("OLD"), e),
            Error::Rhs(e) => (Path::new("NEW"), e),
        };

        let err_str: Cow<str> = match io_error.kind() {
            io::ErrorKind::NotFound => Cow::from("file not found"),
            io::ErrorKind::PermissionDenied => Cow::from("permission denied"),
            io::ErrorKind::Interrupted => Cow::from("file reading was interrupted"),
            io::ErrorKind::InvalidData => Cow::from(format!("invalid data: {}", io_error)),
            _ => Cow::from(format!("unexpected error: {:?}", io_error)),
        };

        let full_path = prefix.join(path);

        writeln!(
            out,
            "{}{} error for file {}: {}{}",
            color_code,
            LineStatus::Error.indicator(),
            PercentPath::from(&full_path),
            err_str,
            self.color_codes.reset
        )
        .unwrap();
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

    pub fn err_no(&self) -> usize {
        self.err_no
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

        self.err_no()
    }

    /// Creates a new `DiffPrinter` which will use `color_codes` to write colored output.
    /// `percent_paths` determines if the paths will be percent-encoded.
    pub fn new(color_codes: LineStatusColorCodes, percent_paths: bool) -> Self {
        Self {
            color_codes,
            percent_paths,
            err_no: 0,
        }
    }
}
