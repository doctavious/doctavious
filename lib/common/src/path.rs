use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// Appends string to the end of the supplied path
pub fn append_to_path(path: impl Into<OsString>, s: impl AsRef<OsStr>) -> PathBuf {
    let mut p = path.into();
    p.push(s);
    p.into()
}
