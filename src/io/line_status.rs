/// `LineStatus` indicates the status of a diff line.
///
/// There is a one-to-one correspondence between it and the one-character indicator printed on the
/// beginning of each line as well as the color the line is printed with (if the output is
/// colored).
#[derive(Copy, Clone)]
pub enum LineStatus {
    Deleted,
    Added,
    Modified,
    Error,
    ErrorDescription,
}

impl LineStatus {
    pub fn indicator(&self) -> char {
        match self {
            LineStatus::Deleted => '-',
            LineStatus::Added => '+',
            LineStatus::Modified => '~',
            LineStatus::Error => '!',
            LineStatus::ErrorDescription => '^',
        }
    }
}
