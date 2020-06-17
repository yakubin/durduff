use std::cmp::Ordering;

use std::collections::VecDeque;

use std::convert::TryFrom;

use std::io;

use std::iter::Iterator;

use std::path::Path;
use std::path::PathBuf;

use crate::io::utf8_percent_encode_path;

/// Compares paths, so that:
/// - paths with a smaller number of components are smaller
/// - paths with equal numbers of components are compared lexicographically
pub fn cmp_paths(lhs: &PathBuf, rhs: &PathBuf) -> Ordering {
    match lhs.components().count().cmp(&rhs.components().count()) {
        Ordering::Equal => lhs.cmp(rhs),
        o => o,
    }
}

/// Recursive directory iterator
///
/// It yields paths in the order given by `cmp_paths`.
pub struct RecDirIter {
    top: PathBuf,
    to_traverse: VecDeque<PathBuf>,
    error: Option<io::Error>,
}

/// How many elements we expect a directory may have at most
///
/// It's just a performance hint. The program won't break in cases where it's not true.
const DIR_ELEMS_MAX: usize = 4 << 10;

fn try_append_dir_elems(dst: &mut VecDeque<PathBuf>, top: &Path, dir: &Path) -> io::Result<()> {
    let full_prefix = top.join(dir);

    if !full_prefix.symlink_metadata()?.file_type().is_dir() {
        return Ok(());
    }

    let mut elems = Vec::with_capacity(DIR_ELEMS_MAX);

    for e in full_prefix.read_dir()? {
        elems.push(e?.path().strip_prefix(top).unwrap().to_path_buf());
    }

    // the following is a lot faster than either sort_unstable, sort_unstable_by_key, or sort_by_key.
    elems.sort_by_cached_key(|p| p.file_name().unwrap().to_os_string());

    dst.extend(elems.drain(..));

    Ok(())
}

/// Replaces the description of `e` with "reading directory " + UTF-8 percent-encoded `p`.
///
/// Needed for reporting in `run_diff`, which directory caused the error.
fn annotate_error(p: &Path, e: io::Error) -> io::Error {
    io::Error::new(
        e.kind(),
        format!("reading directory {}", utf8_percent_encode_path(p)),
    )
}

impl RecDirIter {
    fn try_append_dir_elems(&mut self, dir: &Path) -> io::Result<()> {
        try_append_dir_elems(&mut self.to_traverse, &self.top, dir)
    }

    fn append_dir_elems(&mut self, d: &Path) {
        if let Err(e) = self.try_append_dir_elems(d) {
            let err_path = self.top.join(d);
            self.error = Some(annotate_error(&err_path, e));
        }
    }
}

#[derive(Debug)]
pub struct RecDirIterTopIsNotDir;

impl TryFrom<PathBuf> for RecDirIter {
    type Error = RecDirIterTopIsNotDir;

    fn try_from(top: PathBuf) -> Result<Self, Self::Error> {
        let mut iter = Self {
            top,
            to_traverse: VecDeque::new(),
            error: None,
        };

        let null_path = Path::new("");

        // appease the borrow checker...
        //
        // (writing "&iter.top" in the closure conflicts with the "&mut iter.to_traverse" below.
        // don't ask me why...)
        let top = &iter.top;

        let annot_error = |e: io::Error| annotate_error(&top, e);

        let top_metadata = iter.top.symlink_metadata();

        iter.error = match top_metadata {
            Ok(m) => {
                let ft = m.file_type();

                if !ft.is_dir() {
                    return Err(RecDirIterTopIsNotDir);
                }

                try_append_dir_elems(&mut iter.to_traverse, &iter.top, &null_path)
                    .err()
                    .map(annot_error)
            }
            Err(e) => Some(e),
        };

        Ok(iter)
    }
}

impl Iterator for RecDirIter {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.error.is_some() {
            self.to_traverse.truncate(0);
            return Some(Err(self.error.take().unwrap()));
        }

        if let Some(p) = self.to_traverse.pop_front() {
            self.append_dir_elems(&p);
            Some(Ok(p))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.error.is_none() {
            (self.to_traverse.len(), None)
        } else {
            (0, Some(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test needs to be run in the project root directory.
    #[test]
    fn rudimentary() -> Result<(), io::Error> {
        let pb = |s: &str| PathBuf::from(s.to_string());

        let paths_res: Vec<_> = RecDirIter::try_from(pb("test-data/rudimentary"))
            .unwrap()
            .collect();

        let mut paths = Vec::new();

        for p in paths_res {
            paths.push(p?);
        }

        let expected = vec![
            pb("new"),
            pb("old"),
            pb("new/b"),
            pb("new/c"),
            pb("new/d"),
            pb("new/foo"),
            pb("old/c"),
            pb("old/d"),
            pb("old/foo"),
            pb("new/foo/a"),
            pb("old/foo/a"),
        ];

        assert_eq!(paths, expected);

        Ok(())
    }

    /// This test needs to be run in the project root directory.
    #[test]
    fn root_dir_does_not_exist() {
        let pb = |s: &str| PathBuf::from(s.to_string());

        let mut it = RecDirIter::try_from(pb("xb1suKLrl0Ltenl6T0CgzbI0shecZpXYLmEqzg")).unwrap();

        let item = it.next();

        assert!(item.is_some());

        let result = item.unwrap();

        assert!(result.is_err());

        let err = result.err().unwrap();

        assert_eq!(err.kind(), io::ErrorKind::NotFound);

        assert!(it.next().is_none());
    }

    /// This test needs to be run in the project root directory.
    #[test]
    fn root_dir_is_a_regular_file() {
        let pb = |s: &str| PathBuf::from(s.to_string());

        let r = RecDirIter::try_from(pb("test-data/rudimentary/old/foo/a"));

        assert!(r.is_err());
    }
}
