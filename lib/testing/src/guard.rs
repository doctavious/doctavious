use std::panic::{RefUnwindSafe, UnwindSafe};
use std::path::Path;
use std::sync::Mutex;
use std::{env, io, panic};

pub struct CwdGuard {
    original_dir: std::path::PathBuf,
}

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

// with_directory(path, || { closure })
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
