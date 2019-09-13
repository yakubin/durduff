use std::fmt;

use std::io;
use std::io::Write;

use std::time::Instant;

use super::ManualBufWriter;
use super::ProgressData;

/// Flushes per second.
pub const FPS: u128 = 25;

/// `FpsBufWriter` wraps a writer to provide buffering which flushes with the frequency specified
/// by `FPS`. With each flush progress bar is printed to `progress_out`.
pub struct FpsBufWriter<'a, W: Write> {
    inner: ManualBufWriter<W>,

    last_flush: Instant,

    get_progress_out_width: fn() -> usize,

    progress_out: &'a mut dyn Write,

    progress: ProgressData,
}

impl<'a, W: Write> FpsBufWriter<'a, W> {
    pub fn new(
        inner: W,
        progress_out: &'a mut dyn Write,
        get_progress_out_width: fn() -> usize,
    ) -> Self {
        Self {
            inner: ManualBufWriter::new(inner, 0),
            last_flush: Instant::now(),
            get_progress_out_width,
            progress_out,
            progress: ProgressData::default(),
        }
    }

    fn clear_progress_line(&mut self) -> io::Result<()> {
        if cfg!(unix) {
            self.progress_out.write_all(b"\r\x1B[2K")
        } else {
            let line_width = (self.get_progress_out_width)();
            let line_of_spaces: Vec<u8> = std::iter::repeat(b' ').take(line_width).collect();
            self.progress_out.write_all(&line_of_spaces)?;
            self.progress_out.write_all(b"\r")
        }
    }

    pub fn estimate_more(&mut self, more: usize) {
        self.progress.estimate_more(more);
    }

    pub fn processed(&mut self) {
        self.progress.processed();
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn try_flush(&mut self) -> io::Result<()> {
        let now = Instant::now();

        let ms_since_last_flush = (now - self.last_flush).as_millis();

        if 1000 < ms_since_last_flush * FPS {
            self.flush_now(now, false)
        } else {
            Ok(())
        }
    }

    fn flush_now(&mut self, now: Instant, flush_inner: bool) -> io::Result<()> {
        self.last_flush = now;

        self.clear_progress_line()?;

        self.inner.flush()?;

        if flush_inner {
            self.inner.flush_inner()?;
        }

        self.clear_progress_line()?;
        self.progress
            .print((self.get_progress_out_width)(), &mut self.progress_out)
    }
}

impl<'a, W: Write> Write for FpsBufWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_now(Instant::now(), true)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.inner.write_fmt(fmt)
    }
}
