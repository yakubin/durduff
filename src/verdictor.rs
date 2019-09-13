use std::fs::File;

use std::io::Read;

use std::path::Path;

use crate::error::*;
use crate::io::ReadIntMitigator;
use crate::iter::SumIterSelector;
use crate::verdict::Verdict;

const DEFAULT_BLKSIZE: usize = 512 << 10; // 512 KiB

/// Based on items from `SumIter` gives verdicts whether files at a specified path (with different
/// prefixes) differ.
pub struct Verdictor<'a> {
    lhs_prefix: &'a Path,

    rhs_prefix: &'a Path,

    lhs_buf: Vec<u8>,

    rhs_buf: Vec<u8>,

    blksize: usize,
}

impl<'a> Verdictor<'a> {
    /// Creates a new verdictor.
    pub fn new(
        lhs_prefix: &'a Path,
        rhs_prefix: &'a Path,
        blksize_override: Option<usize>,
    ) -> Verdictor<'a> {
        Verdictor {
            lhs_prefix,
            rhs_prefix,

            lhs_buf: Vec::new(),
            rhs_buf: Vec::new(),

            blksize: blksize_override.unwrap_or(DEFAULT_BLKSIZE),
        }
    }

    /// Compares symlink target paths.
    fn cmp_symlinks(&self, lhs: &Path, rhs: &Path) -> Result<Verdict> {
        let ll = lhs.read_link().wrap_lhs()?;
        let rl = rhs.read_link().wrap_rhs()?;

        if ll == rl {
            Ok(Verdict::Same)
        } else {
            Ok(Verdict::Modified)
        }
    }

    /// Compares the contents of two readers.
    fn cmp_readers<L, R>(&mut self, lhs: L, rhs: R) -> Result<Verdict>
    where
        L: Read,
        R: Read,
    {
        let mut miti_lhs = ReadIntMitigator(lhs);
        let mut miti_rhs = ReadIntMitigator(rhs);

        loop {
            let lhs_bytes_no = miti_lhs.read(&mut self.lhs_buf).wrap_lhs()?;
            let rhs_bytes_no = miti_rhs.read(&mut self.rhs_buf).wrap_rhs()?;

            if lhs_bytes_no != rhs_bytes_no {
                return Ok(Verdict::Modified);
            } else if lhs_bytes_no == 0 {
                return Ok(Verdict::Same);
            } else if self.lhs_buf != self.rhs_buf {
                return Ok(Verdict::Modified);
            }
        }
    }

    /// Compares the contents of files `lhs` and `rhs`.
    fn cmp_contents(&mut self, lhs: &Path, rhs: &Path) -> Result<Verdict> {
        let lhs_file = File::open(&lhs).wrap_lhs()?;
        let rhs_file = File::open(&rhs).wrap_rhs()?;

        self.cmp_readers(lhs_file, rhs_file)
    }

    /// Compares files with the paths ending with suffix `suffix` and beginning with prefixes passed to `new`.
    fn cmp_files(&mut self, suffix: &Path) -> Result<Verdict> {
        let lhs_path = self.lhs_prefix.join(&suffix);
        let rhs_path = self.rhs_prefix.join(&suffix);

        let lhs_metadata = lhs_path.symlink_metadata().wrap_lhs()?;
        let rhs_metadata = rhs_path.symlink_metadata().wrap_rhs()?;

        let lhs_file_type = lhs_metadata.file_type();
        let rhs_file_type = rhs_metadata.file_type();

        let lhs_ftype = (
            lhs_file_type.is_dir(),
            lhs_file_type.is_file(),
            lhs_file_type.is_symlink(),
        );
        let rhs_ftype = (
            rhs_file_type.is_dir(),
            rhs_file_type.is_file(),
            rhs_file_type.is_symlink(),
        );

        if lhs_ftype != rhs_ftype {
            Ok(Verdict::Modified)
        } else if lhs_file_type.is_symlink() {
            self.cmp_symlinks(&lhs_path, &rhs_path)
        } else if lhs_file_type.is_file() {
            if lhs_metadata.len() == rhs_metadata.len() {
                self.lhs_buf = vec![0_u8; self.blksize];
                self.rhs_buf = vec![0_u8; self.blksize];

                self.cmp_contents(&lhs_path, &rhs_path)
            } else {
                Ok(Verdict::Modified)
            }
        } else if lhs_file_type.is_dir() {
            Ok(Verdict::Same)
        } else {
            let err_msg = format!(
                "unrecognized file type for path: {}",
                lhs_path.to_string_lossy()
            );
            Err(Error::Lhs(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                err_msg,
            )))
        }
    }

    /// Compares files with the paths ending with suffix `suffix` and beginning with prefixes passed to `new`.
    /// Relies on the caller to inform through `sel` which of the two files exist.
    pub fn get_verdict(&mut self, sel: &SumIterSelector, path: &Path) -> Result<Verdict> {
        match sel {
            SumIterSelector::Left => Ok(Verdict::Deleted),
            SumIterSelector::Right => Ok(Verdict::Added),
            SumIterSelector::Both => self.cmp_files(path),
        }
    }
}
