use std::fs::File;

use std::io;
use std::io::Read;

use std::path::Path;
use std::path::PathBuf;

use crate::io::ReadIntMitigator;
use crate::iter::SumIterSelector;
use crate::verdict::Verdict;

/// Default block size used to read files
const DEFAULT_BLKSIZE: usize = 512 << 10; // 512 KiB

/// Based on items from `SumIter`, gives verdicts whether files at a specified path (with different
/// prefixes) differ.
pub struct Verdictor<'a> {
    lhs_prefix: &'a Path,

    rhs_prefix: &'a Path,

    lhs_buf: Vec<u8>,

    rhs_buf: Vec<u8>,

    blksize: usize,
}

/// Couples I/O error with path prefix indicating which tree (lhs vs rhs) the error was encountered
/// in.
///
/// The prefix is usually stripped, because it's not printed in the normal output. However, it may
/// be useful when reporting errors.
type PrivError<'a> = (io::ErrorKind, &'a Path);

/// Result based on PrivError
type PrivResult<'a> = Result<Verdict, PrivError<'a>>;

/// Converts `(result, path)` into `(Verdict, PathBuf)` suitable for printing.
fn priv_result_to_ver_path(result: PrivResult, path: PathBuf) -> (Verdict, PathBuf) {
    match result {
        Ok(verdict) => (verdict, path),
        Err((error_kind, prefix)) => (Verdict::Error(error_kind), prefix.join(path)),
    }
}

/// Used to convert `std::io::Result` into `PrivResult`, annotating errors with the path to the
/// directory tree where they were encountered.
trait IoResult<T> {
    fn annotate(self, path: &Path) -> Result<T, PrivError>;
}

impl<T> IoResult<T> for io::Result<T> {
    fn annotate(self, path: &Path) -> Result<T, PrivError> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err((e.kind(), path)),
        }
    }
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
    fn cmp_symlinks(&self, lhs: &Path, rhs: &Path) -> PrivResult<'a> {
        let ll = lhs.read_link().annotate(self.lhs_prefix)?;
        let rl = rhs.read_link().annotate(self.rhs_prefix)?;

        if ll == rl {
            Ok(Verdict::Same)
        } else {
            Ok(Verdict::Modified)
        }
    }

    /// Compares the contents of files `lhs` and `rhs`.
    fn cmp_contents(&mut self, lhs: &Path, rhs: &Path) -> PrivResult<'a> {
        let lhs_file = File::open(&lhs).annotate(self.lhs_prefix)?;
        let rhs_file = File::open(&rhs).annotate(self.rhs_prefix)?;

        let mut miti_lhs = ReadIntMitigator(lhs_file);
        let mut miti_rhs = ReadIntMitigator(rhs_file);

        loop {
            let lhs_bytes_no = miti_lhs.read(&mut self.lhs_buf).annotate(self.lhs_prefix)?;
            let rhs_bytes_no = miti_rhs.read(&mut self.rhs_buf).annotate(self.rhs_prefix)?;

            if lhs_bytes_no != rhs_bytes_no {
                return Ok(Verdict::Modified);
            } else if lhs_bytes_no == 0 {
                return Ok(Verdict::Same);
            } else if self.lhs_buf != self.rhs_buf {
                return Ok(Verdict::Modified);
            }
        }
    }

    /// Compares files with the paths ending with suffix `suffix` and beginning with prefixes
    /// passed to `new`.
    fn cmp_files(&mut self, suffix: &Path) -> PrivResult<'a> {
        let lhs_path = self.lhs_prefix.join(&suffix);
        let rhs_path = self.rhs_prefix.join(&suffix);

        let lhs_metadata = lhs_path.symlink_metadata().annotate(self.lhs_prefix)?;
        let rhs_metadata = rhs_path.symlink_metadata().annotate(self.rhs_prefix)?;

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
            Err((std::io::ErrorKind::InvalidData, self.lhs_prefix))
        }
    }

    /// Compares files with the paths ending with suffix `suffix` and beginning with prefixes
    /// passed to `new`. Relies on the caller to inform through `sel` which of the two files exist.
    pub fn get_verdict(&mut self, (sel, path): (SumIterSelector, PathBuf)) -> (Verdict, PathBuf) {
        match sel {
            SumIterSelector::Left => (Verdict::Deleted, path),
            SumIterSelector::Right => (Verdict::Added, path),
            SumIterSelector::Both => priv_result_to_ver_path(self.cmp_files(&path), path),
        }
    }
}
