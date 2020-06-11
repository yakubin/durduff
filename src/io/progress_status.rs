/// Data necessary to print a progress bar
pub struct ProgressStatus {
    pub total_no: usize,
    pub processed_no: usize,
}

impl ProgressStatus {
    /// `more` hints how many elements are left to be processed after the current one. Hints which
    /// indicate fewer elements than estimated previously will be disregarded.
    pub fn estimate_more(&mut self, more: usize) {
        let new_total_no = self.processed_no + more;

        if self.total_no < new_total_no {
            self.total_no = new_total_no;
        }
    }

    /// Should be called each time an element is processed.
    pub fn processed(&mut self) {
        if self.total_no == self.processed_no {
            self.total_no += 1;
        }

        self.processed_no += 1;
    }
}

impl Default for ProgressStatus {
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
    fn estimate_more() {
        let mut progress = ProgressStatus {
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
        let mut progress = ProgressStatus {
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
