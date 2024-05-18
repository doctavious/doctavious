use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub fn append_to_path(p: impl Into<OsString>, s: impl AsRef<OsStr>) -> PathBuf {
    let mut p = p.into();
    p.push(s);
    p.into()
}
