// This was mostly taken from https://docs.rs/crate/edit/

use std::ffi::OsStr;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

pub use tempfile::Builder;
use which::which;

static ENV_VARS: &[&str] = &["VISUAL", "EDITOR"];

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // CLI editors
    "nvim", "vim", "vi", "emacs", "nano", "pico",
    // GUI editors
    "code", "atom", "subl", "gedit", "gvim",
    // Generic "file openers"
    "xdg-open", "gnome-open", "kde-open",
];

#[cfg(target_os = "macos")]
#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // CLI editors
    "nvim", "vim", "vi", "emacs", "nano", "pico",
    // GUI editors
    "code", "atom", "subl", "gvim", "mate",
    // Generic "file openers"
    "open -a TextEdit",
    "open -a TextMate",
    "open",
];

#[cfg(target_os = "windows")]
#[rustfmt::skip]
static HARDCODED_NAMES: &[&str] = &[
    // GUI editors
    "code.exe", "atom.exe", "subl.exe", "notepad++.exe",
    // Installed by default
    "notepad.exe",
    // Generic "file openers"
    "cmd.exe /C start",
];

fn check_editor<T: AsRef<OsStr>>(binary_name: T) -> bool {
    which(binary_name).is_ok()
}

fn string_to_cmd(s: String) -> (PathBuf, Vec<String>) {
    let mut args = s.split_ascii_whitespace();
    (
        args.next().unwrap().into(),
        args.map(String::from).collect(),
    )
}

fn get_editor_args() -> Result<(PathBuf, Vec<String>)> {
    ENV_VARS
        .iter()
        .filter_map(env::var_os)
        .filter(|v| !v.is_empty())
        .filter_map(|v| v.into_string().ok())
        .map(string_to_cmd)
        .filter(|(p, _)| check_editor(p))
        .next()
        .or_else(|| {
            HARDCODED_NAMES
                .iter()
                .map(|s| s.to_string())
                .map(string_to_cmd)
                .filter(|(p, _)| check_editor(p))
                .next()
        })
        .ok_or_else(|| Error::from(ErrorKind::NotFound))
}

/// Find the system default editor, if there is one.
///
/// This function checks several sources to find an editor binary (in order of precedence):
///
/// - the `VISUAL` environment variable
/// - the `EDITOR` environment variable
/// - hardcoded lists of common CLI editors on MacOS/Unix
/// - hardcoded lists of GUI editors on Windows/MacOS/Unix
/// - platform-specific generic "file openers" (e.g. `xdg-open` on Linux and `open` on MacOS)
///
/// Also, it doesn't blindly return whatever is in an environment variable.
/// If a specified editor can't be found or isn't marked as executable this function will fall back to the next one that is.
pub fn get_editor() -> Result<PathBuf> {
    get_editor_args().map(|(x, _)| x)
}

/// Open the contents of a string or buffer in the [default editor].
///
/// This function saves its input to a temporary file and then opens the default editor to it.
/// It waits for the editor to return, re-reads the (possibly changed/edited) temporary file, and then deletes it.
pub fn edit<S: AsRef<[u8]>>(text: S) -> Result<String> {
    let builder = Builder::new();
    edit_with_builder(text, &builder)
}

/// Open the contents of a string or buffer in the [default editor] using a temporary file with a
/// custom path or filename.
///
/// This function saves its input to a temporary file created using `builder`, then opens the
/// default editor to it. It waits for the editor to return, re-reads the (possibly changed/edited)
/// temporary file, and then deletes it.
///
/// Other than the custom [`Builder`], this function is identical to [`edit`].
///
/// # Arguments
///
/// `builder` is used to create a temporary file, potentially with a custom name, path, or prefix.
///
/// `text` is written to the temporary file before invoking the editor. (The editor opens with
/// the contents of `text` already in the file).
///
/// # Returns
///
/// If successful, returns the edited string.
/// If the temporary file can't be created with the provided builder, may return any error returned
/// by [`OpenOptions::open`].
/// If the edited version of the file can't be decoded as UTF-8, returns [`ErrorKind::InvalidData`].
/// If no text editor could be found, returns [`ErrorKind::NotFound`].
/// Any errors related to spawning the editor process will also be passed through.
///
/// [default editor]: fn.get_editor.html
/// [`edit`]: fn.edit.html
/// [`Builder`]: struct.Builder.html
/// [`OpenOptions::open`]: https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#errors
/// [`ErrorKind::InvalidData`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.InvalidData
/// [`ErrorKind::NotFound`]: https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.NotFound
pub fn edit_with_builder<S: AsRef<[u8]>>(text: S, builder: &Builder) -> Result<String> {
    String::from_utf8(edit_bytes_with_builder(text, builder)?)
        .map_err(|_| Error::from(ErrorKind::InvalidData))
}

/// Open the contents of a string or buffer in the [default editor] and return them as raw bytes.
///
/// See [`edit`], the version of this function that takes and returns [`String`].
///
/// # Arguments
///
/// `buf` is written to the temporary file before invoking the editor.
///
/// # Returns
///
/// If successful, returns the contents of the temporary file in raw (`Vec<u8>`) form.
///
/// [default editor]: fn.get_editor.html
/// [`edit`]: fn.edit.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
pub fn edit_bytes<B: AsRef<[u8]>>(buf: B) -> Result<Vec<u8>> {
    let builder = Builder::new();
    edit_bytes_with_builder(buf, &builder)
}

/// Open the contents of a string or buffer in the [default editor] using a temporary file with a
/// custom path or filename and return them as raw bytes.
///
/// See [`edit_with_builder`], the version of this function that takes and returns [`String`].
///
/// Other than the custom [`Builder`], this function is identical to [`edit_bytes`].
///
/// # Arguments
///
/// `builder` is used to create a temporary file, potentially with a custom name, path, or prefix.
///
/// `buf` is written to the temporary file before invoking the editor.
///
/// # Returns
///
/// If successful, returns the contents of the temporary file in raw (`Vec<u8>`) form.
///
/// [default editor]: fn.get_editor.html
/// [`edit_with_builder`]: fn.edit_with_builder.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [`Builder`]: struct.Builder.html
/// [`edit_bytes`]: fn.edit_bytes.html
pub fn edit_bytes_with_builder<B: AsRef<[u8]>>(buf: B, builder: &Builder) -> Result<Vec<u8>> {
    let mut file = builder.tempfile()?;
    file.write(buf.as_ref())?;

    let path = file.into_temp_path();
    edit_file(&path)?;

    let edited = fs::read(&path)?;

    path.close()?;
    Ok(edited)
}

/// Open an existing file (or create a new one, depending on the editor's behavior) in the
/// [default editor] and wait for the editor to exit.
///
/// # Arguments
///
/// A [`Path`] to a file, new or existing, to open in the default editor.
///
/// # Returns
///
/// A Result is returned in case of errors finding or spawning the editor, but the contents of the
/// file are not read and returned as in [`edit`] and [`edit_bytes`].
///
/// [default editor]: fn.get_editor.html
/// [`Path`]: https://doc.rust-lang.org/std/path/struct.Path.html
/// [`edit`]: fn.edit.html
/// [`edit_bytes`]: fn.edit_bytes.html
pub fn edit_file<P: AsRef<Path>>(file: P) -> Result<()> {
    let (editor, args) = get_editor_args()?;
    let status = Command::new(&editor)
        .args(&args)
        .arg(file.as_ref())
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?
        .status;

    if status.success() {
        Ok(())
    } else {
        let full_command = if args.is_empty() {
            format!(
                "{} {}",
                editor.to_string_lossy(),
                file.as_ref().to_string_lossy()
            )
        } else {
            format!(
                "{} {} {}",
                editor.to_string_lossy(),
                args.join(" "),
                file.as_ref().to_string_lossy()
            )
        };

        Err(Error::new(
            ErrorKind::Other,
            format!("editor '{}' exited with error: {}", full_command, status),
        ))
    }
}
