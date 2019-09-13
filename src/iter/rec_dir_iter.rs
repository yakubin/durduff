use std::cmp::Ordering;

use std::collections::VecDeque;

use std::io;

use std::iter::Iterator;

use std::path::Path;
use std::path::PathBuf;

/// Compares paths, so that:
/// - paths with a smaller number of components are smaller
/// - paths with equal numbers of components are compared lexicographically
pub fn cmp_paths(lhs: &PathBuf, rhs: &PathBuf) -> Ordering {
    match lhs.components().count().cmp(&rhs.components().count()) {
        Ordering::Equal => lhs.cmp(rhs),
        o => o,
    }
}

/// Recursive directory iterator.
/// It yields paths in the order given by `cmp_paths`.
pub struct RecDirIter {
    top: PathBuf,
    to_traverse: VecDeque<PathBuf>,
    error: Option<io::Error>,
}

fn try_append_dir_elems(dst: &mut VecDeque<PathBuf>, top: &Path, dir: &Path) -> io::Result<()> {
    let full_prefix = top.join(dir);

    if !full_prefix.symlink_metadata()?.file_type().is_dir() {
        return Ok(());
    }

    let mut elems = Vec::new();

    for e in full_prefix.read_dir()? {
        elems.push(e?.path().strip_prefix(&full_prefix).unwrap().to_path_buf());
    }

    elems.sort_unstable();

    dst.extend(elems.drain(..).map(|p| dir.join(p)));

    Ok(())
}

impl RecDirIter {
    fn try_append_dir_elems(&mut self, dir: &Path) -> io::Result<()> {
        try_append_dir_elems(&mut self.to_traverse, &self.top, dir)
    }

    fn append_dir_elems(&mut self, d: &Path) {
        if let Err(e) = self.try_append_dir_elems(d) {
            self.error = Some(e);
        }
    }
}

impl From<PathBuf> for RecDirIter {
    fn from(top: PathBuf) -> Self {
        let mut iter = Self {
            top,
            to_traverse: VecDeque::new(),
            error: None,
        };

        let null_path = Path::new("");

        if let Err(e) = try_append_dir_elems(&mut iter.to_traverse, &iter.top, &null_path) {
            eprintln!("error in constructor. top: {}", iter.top.to_string_lossy());
            iter.error = Some(e);
        }

        iter
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

        let paths_res: Vec<_> = RecDirIter::from(pb("test-data/rudimentary")).collect();

        let mut paths = Vec::new();

        for p in paths_res {
            paths.push(p?);
        }

        let expected = vec![
            pb("func.test"),
            pb("new"),
            pb("old"),
            pb("new/b"),
            pb("new/c"),
            pb("new/d"),
            pb("new/foo"),
            pb("old/bar"),
            pb("old/c"),
            pb("old/d"),
            pb("old/foo"),
            pb("new/foo/a"),
            pb("new/foo/baz"),
            pb("old/foo/a"),
            pb("old/foo/baz"),
        ];

        assert_eq!(paths, expected);

        Ok(())
    }

    /// This test needs to be run in the project root directory.
    #[test]
    fn root_dir_does_not_exist() {
        let pb = |s: &str| PathBuf::from(s.to_string());

        let mut it = RecDirIter::from(pb("xb1suKLrl0Ltenl6T0CgzbI0shecZpXYLmEqzg"));

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

        let mut it = RecDirIter::from(pb("test-data/rudimentary/old/foo/a"));

        let item = it.next();

        assert!(item.is_some());

        let result = item.unwrap();

        assert!(result.is_err());

        let err = result.err().unwrap();

        assert_eq!(err.kind(), io::ErrorKind::Other);

        assert!(it.next().is_none());
    }
}
