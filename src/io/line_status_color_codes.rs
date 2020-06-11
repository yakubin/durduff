use super::color_codes::*;
use super::LineStatus;

/// Color codes used to print diff lines of different statuses (see
/// `LineStatus`)
#[derive(Clone)]
pub struct LineStatusColorCodes {
    pub deleted: &'static [u8],
    pub added: &'static [u8],
    pub modified: &'static [u8],
    pub error: &'static [u8],

    /// Resets the foreground color to its original value.
    pub reset: &'static [u8],
}

impl LineStatusColorCodes {
    pub fn no_color() -> Self {
        Self {
            deleted: b"",
            added: b"",
            modified: b"",
            error: b"",
            reset: b"",
        }
    }

    pub fn color() -> Self {
        Self {
            deleted: YELLOW,
            added: GREEN,
            modified: BLUE,
            error: RED,
            reset: RESET,
        }
    }

    pub fn get(&self, status: LineStatus) -> &'static [u8] {
        match status {
            LineStatus::Deleted => self.deleted,
            LineStatus::Added => self.added,
            LineStatus::Modified => self.modified,
            LineStatus::Error => self.error,
            LineStatus::ErrorDescription => self.error,
        }
    }
}
