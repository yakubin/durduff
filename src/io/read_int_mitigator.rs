use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;

/// Reader wrapper which mitigates problems with incomplete or interrupted reads
pub struct ReadIntMitigator<R: Read>(pub R);

impl<R: Read> Read for ReadIntMitigator<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let Self(ref mut reader) = self;

        let mut to_read = buf.len();

        let mut last_interrupted = false;

        while 0 < to_read {
            let offset = buf.len() - to_read;

            match reader.read(&mut buf[offset..]) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    to_read -= n;
                    last_interrupted = false;
                }
                Err(e) => {
                    if e.kind() == ErrorKind::Interrupted && !last_interrupted {
                        last_interrupted = true;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Ok(buf.len() - to_read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cmp::min;
    use std::io::Error;

    struct InterruptingReader {
        content: Vec<u8>,

        pos: usize,

        max_len: usize,

        interrupt_no: usize,
    }

    impl Read for InterruptingReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            if 0 < self.interrupt_no {
                self.interrupt_no -= 1;
                return Err(Error::new(ErrorKind::Interrupted, "forced interrupt"));
            }

            let len = min(min(buf.len(), self.content.len() - self.pos), self.max_len);

            let target_slice = &mut buf[..len];

            target_slice.copy_from_slice(self.content.get(self.pos..self.pos + len).unwrap());

            self.pos += len;

            Ok(len)
        }
    }

    const CONTENT: std::ops::Range<u8> = 0..128;

    #[test]
    fn single_interrupt() -> Result<()> {
        let content: Vec<_> = CONTENT.collect();

        let reader = InterruptingReader {
            content: content.clone(),
            pos: 0,
            max_len: content.len(),
            interrupt_no: 1,
        };

        let mut mitigator = ReadIntMitigator(reader);

        let mut buf = vec![0; content.len()];

        let read_no = mitigator.read(buf.as_mut_slice())?;

        assert_eq!(read_no, content.len());
        assert_eq!(buf, content);

        Ok(())
    }

    #[test]
    fn two_interrupts() {
        let content: Vec<_> = CONTENT.collect();

        let reader = InterruptingReader {
            content: content.clone(),
            pos: 0,
            max_len: content.len(),
            interrupt_no: 2,
        };

        let mut mitigator = ReadIntMitigator(reader);

        let mut buf = vec![0; content.len()];

        let result = mitigator.read(buf.as_mut_slice());

        assert!(result.is_err());

        let err = result.err().unwrap();

        assert_eq!(err.kind(), ErrorKind::Interrupted);
    }

    #[test]
    fn incomplete_reads() -> Result<()> {
        let content: Vec<_> = CONTENT.collect();

        assert!(40 < content.len());

        let reader = InterruptingReader {
            content: content.clone(),
            pos: 0,
            max_len: 40,
            interrupt_no: 0,
        };

        let mut mitigator = ReadIntMitigator(reader);

        let mut buf = vec![0; content.len()];

        let read_no = mitigator.read(buf.as_mut_slice())?;

        assert_eq!(read_no, content.len());
        assert_eq!(buf, content);

        Ok(())
    }

    #[test]
    fn incomplete_reads_with_single_interrupt() -> Result<()> {
        let content: Vec<_> = CONTENT.collect();

        assert!(40 < content.len());

        let reader = InterruptingReader {
            content: content.clone(),
            pos: 0,
            max_len: 40,
            interrupt_no: 1,
        };

        let mut mitigator = ReadIntMitigator(reader);

        let mut buf = vec![0; content.len()];

        let read_no = mitigator.read(buf.as_mut_slice())?;

        assert_eq!(read_no, content.len());
        assert_eq!(buf, content);

        Ok(())
    }

    #[test]
    fn incomplete_reads_with_two_interrupts() {
        let content: Vec<_> = CONTENT.collect();

        assert!(40 < content.len());

        let reader = InterruptingReader {
            content: content.clone(),
            pos: 0,
            max_len: 40,
            interrupt_no: 2,
        };

        let mut mitigator = ReadIntMitigator(reader);

        let mut buf = vec![0; content.len()];

        let result = mitigator.read(buf.as_mut_slice());

        assert!(result.is_err());

        let err = result.err().unwrap();

        assert_eq!(err.kind(), ErrorKind::Interrupted);
    }
}
