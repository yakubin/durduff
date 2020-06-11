//! VT100 ANSI escape codes for foreground colors

pub const BLACK: &[u8] = b"\x1B[30m";
pub const RED: &[u8] = b"\x1B[31m";
pub const GREEN: &[u8] = b"\x1B[32m";
pub const YELLOW: &[u8] = b"\x1B[33m";
pub const BLUE: &[u8] = b"\x1B[34m";
pub const MAGENTA: &[u8] = b"\x1B[35m";
pub const CYAN: &[u8] = b"\x1B[36m";
pub const WHITE: &[u8] = b"\x1B[37m";
pub const RESET: &[u8] = b"\x1B[39m";
