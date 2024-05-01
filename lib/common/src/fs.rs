use std::path::Path;
use std::{fs, io};
use std::io::{Error, ErrorKind};

// TODO: I feel like I could do this better
pub fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    if !src.as_ref().is_dir() {
        // TODO (sean): change to ErrorKind::NotADirectory when io_error_more stabilizes
        return Err(Error::new(ErrorKind::Other, "Not a directory"));
    }

    let dest = if src.as_ref().to_string_lossy().ends_with("/") {
        dst.as_ref().to_path_buf()
    } else {
        dst.as_ref().join(src.as_ref().file_name().unwrap())
    };

    recursive_dir_copy(src, dst)?;

    Ok(())
}

fn recursive_dir_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            recursive_dir_copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
