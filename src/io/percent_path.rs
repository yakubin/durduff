use std::borrow::Borrow;

use std::ffi::OsStr;

use std::fmt;
use std::fmt::Display;

use std::path::Path;

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};

/// <https://url.spec.whatwg.org/#path-percent-encode-set>
const PATH_PERCENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'#')
    .add(b'?')
    .add(b'{')
    .add(b'}');

/// Wrapper for `Path` which implements the `Display` trait, printing the path using the
/// utf-8 percent encoding with the path percent-encode set.
/// See: <https://url.spec.whatwg.org/#utf-8-percent-encode>
#[derive(Debug)]
pub struct PercentPath<'a> {
    path: &'a Path,
    path_str: Result<&'a str, String>,
}

impl<'a> PercentPath<'a> {
    pub fn is_utf8(&self) -> bool {
        self.path_str.is_ok()
    }
}

impl<'a> AsRef<OsStr> for PercentPath<'a> {
    fn as_ref(&self) -> &OsStr {
        self.path.as_ref()
    }
}

impl<'a> AsRef<Path> for PercentPath<'a> {
    fn as_ref(&self) -> &Path {
        self.path
    }
}

impl<'a> AsRef<PercentPath<'a>> for PercentPath<'a> {
    fn as_ref(&self) -> &PercentPath<'a> {
        self
    }
}

impl<'a> Borrow<Path> for PercentPath<'a> {
    fn borrow(&self) -> &Path {
        self.path
    }
}

impl<'a> Display for PercentPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self.path_str {
            Ok(s) => s,
            Err(ref s) => s,
        };

        utf8_percent_encode(s, PATH_PERCENT_ENCODE_SET).fmt(f)
    }
}

impl<'a, P: AsRef<Path> + ?Sized> From<&'a P> for PercentPath<'a> {
    fn from(p: &'a P) -> PercentPath<'a> {
        let pr = p.as_ref();

        let s = match pr.to_str() {
            Some(s) => Ok(s),
            None => Err(pr.to_string_lossy().into_owned()),
        };

        PercentPath {
            path: pr,
            path_str: s,
        }
    }
}
