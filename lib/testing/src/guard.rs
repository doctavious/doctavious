use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::{Path, PathBuf};
use std::{env, fs, io};

use tempfile::TempDir;

pub struct CwdGuard {
    original_dir: PathBuf,
}

// TODO: this needs more thought (and probably a mutex) to be usable
impl CwdGuard {
    pub fn new<P: AsRef<Path>>(new_dir: P) -> io::Result<Self> {
        let original_dir = env::current_dir()?;
        env::set_current_dir(&new_dir)?;
        Ok(Self { original_dir })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}

// TODO: not sure this Result will work
pub fn with_cwd_guard<P, F, R>(path: &P, closure: F) -> io::Result<R>
where
    P: AsRef<Path>,
    F: Fn() -> io::Result<R> + UnwindSafe + RefUnwindSafe,
{
    let _guard = CwdGuard::new(path)?;
    closure()
}

pub struct TempDirGuard {
    pub path: PathBuf,
}

impl TempDirGuard {
    pub fn new() -> io::Result<(PathBuf, Self)> {
        let temp_dir = TempDir::new()?;
        Self::from_tempdir(temp_dir)
    }

    pub fn from_tempdir(temp_dir: TempDir) -> io::Result<(PathBuf, Self)> {
        let path = temp_dir.keep();
        Ok((path.clone(), Self { path }))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// static SERIAL_TEST: Lazy<Mutex<()>> = Lazy::new(Default::default);
// pub fn with_dir<P, F, R>(path: &P, closure: F) -> io::Result<R>
// where
//     P: AsRef<Path>,
//     F: Fn() -> io::Result<R> + UnwindSafe + RefUnwindSafe,
// {
//     let guard = SERIAL_TEST.lock().unwrap();
//     let original_dir = env::current_dir()?;
//     match env::set_current_dir(path) {
//         Ok(_) => {
//             println!("success");
//         }
//         Err(e) => {
//             println!("error...{:?}", e);
//         }
//     }
//
//     // println!("current...{:?}", env::current_dir()?);
//     let a = match panic::catch_unwind(closure) {
//         Ok(result) => {
//             println!("original dir...{:?}", original_dir);
//             env::set_current_dir(original_dir)?;
//             // drop(path); // not sure if we need do drop this here
//             result
//         }
//         Err(err) => {
//             println!("error occurred original dir...{:?}", original_dir);
//             env::set_current_dir(original_dir)?;
//             // drop(path);
//             drop(guard);
//             panic::resume_unwind(err);
//         }
//     };
//     a
// }
