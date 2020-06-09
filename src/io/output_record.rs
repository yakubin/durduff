pub struct OutputRecord {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl OutputRecord {
    pub fn empty() -> OutputRecord {
        OutputRecord {
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }
}
