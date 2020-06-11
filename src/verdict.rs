use std::io::ErrorKind;

/// Verdict (whether a file is changed)
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Verdict {
    Same,
    Deleted,
    Added,
    Modified,
    Error(ErrorKind),
}
