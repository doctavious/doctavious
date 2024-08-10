use lazy_static::lazy_static;
use regex::Regex;

pub mod commit;
pub mod drivers;
pub mod errors;
pub mod hooks;
pub mod providers;

pub const GIT: &str = "git";
pub const HG: &str = "hg";
pub const SVN: &str = "svn";
