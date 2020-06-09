use std::fmt;

use std::io;
use std::io::Write;

/// `ManualBufWriter` provides manual writer buffering. It writes data to an in-memory buffer and
/// flushes the data (writes it to the inner writer) only when the caller calls
/// `ManualBufWriter::flush`.
pub struct ManualBufWriter<W: Write> {
    inner: W,
    buf: Vec<u8>,
}

impl<W: Write> ManualBufWriter<W> {
    pub fn new(inner: W, capacity: usize) -> Self {
        Self {
            inner,
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }
}

impl<W: Write> Write for ManualBufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        <Self as Write>::write_all(self, buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.write_all(self.buf.as_slice())?;
        self.buf.clear();
        self.inner.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.buf.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments) -> io::Result<()> {
        self.buf.write_fmt(fmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rudimentary() -> io::Result<()> {
        let mut writer = ManualBufWriter::new(Vec::<u8>::new(), 0);

        writer.write_all(b"hello, friend")?;

        assert_eq!(writer.inner.as_slice(), b"");

        writer.flush()?;

        assert_eq!(writer.inner.as_slice(), b"hello, friend");

        Ok(())
    }
}
