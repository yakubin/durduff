//! VT100 ANSI escape codes for foreground colors.

cfg_if! {
    if #[cfg(unix)] {
        pub const BLACK:   &str = "\x1B[38;5;0m";
        pub const RED:     &str = "\x1B[38;5;1m";
        pub const GREEN:   &str = "\x1B[38;5;2m";
        pub const YELLOW:  &str = "\x1B[38;5;3m";
        pub const BLUE:    &str = "\x1B[38;5;4m";
        pub const MAGENTA: &str = "\x1B[38;5;5m";
        pub const CYAN:    &str = "\x1B[38;5;6m";
        pub const WHITE:   &str = "\x1B[38;5;7m";
        pub const RESET:   &str = "\x1B[39m";
    } else {
        pub const BLACK:   &str = "";
        pub const RED:     &str = "";
        pub const GREEN:   &str = "";
        pub const YELLOW:  &str = "";
        pub const BLUE:    &str = "";
        pub const MAGENTA: &str = "";
        pub const CYAN:    &str = "";
        pub const WHITE:   &str = "";
        pub const RESET:   &str = "";
    }
}
