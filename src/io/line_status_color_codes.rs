use super::color_codes::*;
use super::LineStatus;

/// `LineStatusColorCodes` are the color codes used to print diff lines of different statuses (see
/// `LineStatus`). `reset` is the escape code resetting the color to its original value.
#[derive(Clone)]
pub struct LineStatusColorCodes {
    pub deleted: &'static [u8],
    pub added: &'static [u8],
    pub modified: &'static [u8],
    pub error: &'static [u8],
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
