/// Verdict (whether a file is changed).
#[derive(PartialEq, Eq)]
pub enum Verdict {
    Same,
    Deleted,
    Added,
    Modified,
}
