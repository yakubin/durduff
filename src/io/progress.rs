use std::io;
use std::io::Write;

/// Data necessary to print a progress bar.
pub struct ProgressData {
    pub total_no: usize,
    pub processed_no: usize,
}

impl ProgressData {
    /// Prints the progress bar to `out`, assuming `out` is attached to a terminal of the width
    /// `width`.
    pub fn print<W: Write>(&self, width: usize, out: &mut W) -> io::Result<()> {
        let progress_percent = if self.total_no == 0 {
            100
        } else {
            self.processed_no * 100 / self.total_no
        };

        let msg = format!(
            "Files processed: {}/{} ({}%)",
            self.processed_no, self.total_no, progress_percent
        );

        if msg.len() < width + 10 {
            let bar_width = width - msg.len() - " []".len();
            let hash_no = if self.total_no == 0 {
                bar_width
            } else {
                self.processed_no * bar_width / self.total_no
            };
            let hash_str: String = std::iter::repeat('#').take(hash_no).collect();
            let padding: String = std::iter::repeat(' ').take(bar_width - hash_no).collect();
            write!(out, "{} [{}{}]", msg, hash_str, padding)
        } else {
            write!(out, "{}", msg)
        }
    }

    pub fn estimate_more(&mut self, more: usize) {
        let new_total_no = self.processed_no + more;

        if self.total_no < new_total_no {
            self.total_no = new_total_no;
        }
    }

    pub fn processed(&mut self) {
        if self.total_no == self.processed_no {
            self.total_no += 1;
        }

        self.processed_no += 1;
    }
}

impl Default for ProgressData {
    fn default() -> Self {
        Self {
            total_no: 0,
            processed_no: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_progress() -> io::Result<()> {
        let progress = ProgressData {
            processed_no: 7,
            total_no: 11,
        };

        let mut buf = Vec::new();

        progress.print(40, &mut buf)?;

        let printed = String::from_utf8(buf).unwrap();

        assert_eq!(printed, "Files processed: 7/11 (63%) [######    ]");

        Ok(())
    }

    #[test]
    fn estimate_more() {
        let mut progress = ProgressData {
            processed_no: 7,
            total_no: 11,
        };

        progress.estimate_more(3);

        assert_eq!(progress.processed_no, 7);
        assert_eq!(progress.total_no, 11);

        progress.estimate_more(4);

        assert_eq!(progress.processed_no, 7);
        assert_eq!(progress.total_no, 11);

        progress.estimate_more(5);

        assert_eq!(progress.processed_no, 7);
        assert_eq!(progress.total_no, 12);

        progress.estimate_more(6);

        assert_eq!(progress.processed_no, 7);
        assert_eq!(progress.total_no, 13);

        progress.processed_no = 13;
        progress.estimate_more(1);

        assert_eq!(progress.processed_no, 13);
        assert_eq!(progress.total_no, 14);
    }

    #[test]
    fn increment_total() {
        let mut progress = ProgressData {
            processed_no: 9,
            total_no: 11,
        };

        progress.processed();

        assert_eq!(progress.processed_no, 10);
        assert_eq!(progress.total_no, 11);

        progress.processed();

        assert_eq!(progress.processed_no, 11);
        assert_eq!(progress.total_no, 11);

        progress.processed();

        assert_eq!(progress.processed_no, 12);
        assert_eq!(progress.total_no, 12);

        progress.processed();

        assert_eq!(progress.processed_no, 13);
        assert_eq!(progress.total_no, 13);
    }
}
