use super::color_codes::*;
use super::LineStatus;

/// `LineStatusColorCodes` are the color codes used to print diff lines of different statuses (see
/// `LineStatus`). `reset` is the escape code resetting the color to its original value.
pub struct LineStatusColorCodes {
    pub deleted: &'static str,
    pub added: &'static str,
    pub modified: &'static str,
    pub error: &'static str,
    pub reset: &'static str,
}

impl LineStatusColorCodes {
    pub fn no_color() -> Self {
        Self {
            deleted: "",
            added: "",
            modified: "",
            error: "",
            reset: "",
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

    pub fn get(&self, status: &LineStatus) -> &'static str {
        match status {
            LineStatus::Deleted => self.deleted,
            LineStatus::Added => self.added,
            LineStatus::Modified => self.modified,
            LineStatus::Error => self.error,
        }
    }
}
