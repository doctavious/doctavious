use std::fmt::{Display, Formatter, Pointer};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use markup::{MARKUP_FORMAT_EXTENSIONS, MarkupFormat};
use scm::drivers::{Scm, ScmRepository};
use thiserror::Error;
use unidecode::unidecode;
use walkdir::{DirEntry, WalkDir};

use crate::errors::{CliResult, DoctaviousCliError};
use crate::file_structure::FileStructure;
use crate::settings::DEFAULT_TEMPLATE_DIR;

pub mod adr;
pub mod rfd;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DesignDecisionErrors {
    #[error("design doc directory already initialized")]
    DesignDocAlreadyInitialized,

    #[error("design doc directory already exists")]
    DesignDocDirectoryAlreadyExists,

    #[error("invalid design doc directory. Should be utf-8")]
    DesignDocDirectoryInvalid,

    #[error("invalid link reference")]
    InvalidLinkReference,

    /// Error that may occur while reserving ADR/RFD number.
    #[error("{0} has already been reserved")]
    ReservedNumberError(u32),

    #[error("Unknown design document: {0}")]
    UnknownDesignDocument(String),
}

#[remain::sorted]
#[derive(Debug, Clone)]
pub enum LinkReference {
    FileName(String),
    Number(u32),
    Path(PathBuf),
}

impl FromStr for LinkReference {
    type Err = DoctaviousCliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Ok(num) = s.parse::<u32>() {
            Self::Number(num)
        } else {
            if let Ok(path) = PathBuf::from_str(s) {
                if path.is_file() {
                    return Ok(Self::Path(path));
                }
            }
            Self::FileName(s.to_string())
        })
    }
}

impl LinkReference {
    pub fn get_record(&self, cwd: &Path) -> Option<PathBuf> {
        let reference = match self {
            Self::FileName(file) => file.to_string(),
            Self::Number(num) => format_number(num),
            Self::Path(path) => return Some(path.to_owned()),
        };

        get_records(cwd)
            .filter_map(|e| {
                if e.path().to_string_lossy().contains(&reference) {
                    Some(e.path().to_path_buf())
                } else {
                    None
                }
            })
            .next()
    }
}

impl Display for LinkReference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::FileName(f) => f.to_string(),
            Self::Number(n) => n.to_string(),
            Self::Path(p) => p.to_string_lossy().to_string(),
        };

        write!(f, "{s}")
    }
}

pub(crate) fn list(dir: &Path, format: MarkupFormat) -> CliResult<Vec<PathBuf>> {
    let mut paths: Vec<_> = get_records(dir)
        .filter(|e| {
            if let Some(extension) = e.path().extension() {
                return extension.to_string_lossy() == format.extension();
            }
            false
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(paths)
}

fn get_records(cwd: &Path) -> impl Iterator<Item = DirEntry> {
    WalkDir::new(cwd)
        .into_iter()
        .filter_entry(|e| {
            if e.path().is_dir() {
                return e.file_name().to_string_lossy() != DEFAULT_TEMPLATE_DIR;
            }

            true
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_valid_file(&e.path()))
}

pub(crate) fn format_number(number: &u32) -> String {
    format!("{:0>4}", number)
}

// TODO: is there a more concise way to do this?
pub(crate) fn build_path(
    dir: &Path,
    title: &str,
    reserved_number: &str,
    extension: MarkupFormat,
    file_structure: FileStructure,
) -> PathBuf {
    match file_structure {
        FileStructure::Flat => {
            let slug = slugify(&title);
            let file_name = format!("{}-{}", reserved_number, slug);
            Path::new(dir)
                .join(file_name)
                .with_extension(extension.to_string())
        }

        FileStructure::Nested => Path::new(dir)
            .join(&reserved_number)
            .join("README")
            .with_extension(extension.to_string()),
    }
}

pub(crate) fn reserve_number(
    dir: &Path,
    number: Option<u32>,
    file_structure: FileStructure,
) -> CliResult<u32> {
    if let Some(i) = number {
        if is_number_reserved(dir, i, file_structure) {
            // TODO: the prompt to overwrite be here?
            eprintln!("{} has already been reserved in directory {:?}", i, dir);
            return Err(DoctaviousCliError::DesignDecisionErrors(
                DesignDecisionErrors::ReservedNumberError(i),
            ));
        }
        Ok(i)
    } else {
        Ok(get_next_number(dir, file_structure))
    }
}

pub(crate) fn is_number_reserved(dir: &Path, number: u32, file_structure: FileStructure) -> bool {
    get_allocated_numbers(dir, file_structure).contains(&number)
}

pub(crate) fn get_allocated_numbers(dir: &Path, file_structure: FileStructure) -> Vec<u32> {
    match file_structure {
        FileStructure::Flat => get_allocated_numbers_via_flat_files(dir),
        FileStructure::Nested => get_allocated_numbers_via_nested(dir),
    }
}

// TODO: do we want a ReservedNumber type?
// TODO: would be nice to do this via an Iterator but having trouble with empty
// expected struct `std::iter::Map`, found struct `std::iter::Empty`
// using vec for now
pub(crate) fn get_allocated_numbers_via_nested(dir: &Path) -> Vec<u32> {
    match fs::read_dir(dir) {
        Ok(files) => {
            return files
                .filter_map(Result::ok)
                .filter_map(|e| {
                    if e.path().is_dir() {
                        e.file_name().to_string_lossy().parse::<u32>().ok()
                    } else {
                        None
                    }
                })
                .collect();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Vec::new();
        }
        Err(e) => {
            // TODO: dont panic here
            panic!("Error reading directory {:?}. Error: {}", dir, e);
        }
    }
}

// TODO: extract this to list or a list helper
// TODO: would be nice to do this via an Iterator but having trouble with empty
// expected struct `std::iter::Map`, found struct `std::iter::Empty`
// using vec for now
pub(crate) fn get_allocated_numbers_via_flat_files(dir: &Path) -> Vec<u32> {
    let mut allocated_numbers = Vec::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| {
            if e.path().is_dir() {
                return e.file_name().to_string_lossy() != DEFAULT_TEMPLATE_DIR;
            }

            true
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_valid_file(&e.path()))
    {
        // The only way I can get this to pass the borrow checker is first mapping
        // to file_name and then doing the rest.
        // I'm probably doing this wrong and should review later
        let file_name = entry.file_name();
        let ss = file_name.to_str().unwrap();
        let first_space_index = ss.find("-").expect("didnt find a hyphen");
        let num: String = ss.chars().take(first_space_index).collect();
        allocated_numbers.push(num.parse::<u32>().unwrap());
    }

    allocated_numbers
}

pub(crate) fn get_next_number(dir: &Path, file_structure: FileStructure) -> u32 {
    // TODO: revisit iterator
    // get_allocated_numbers(dir, file_structure)
    //     .iter()
    //     .max()
    //     .unwrap_or_default() + 1

    if let Some(max) = get_allocated_numbers(dir, file_structure).iter().max() {
        max + 1
    } else {
        1
    }
}

fn can_reserve(repo: &Scm, number: u32) -> CliResult<bool> {
    if repo.is_dirty()? {
        // TODO: return error
    }

    match repo {
        Scm::Git(_) => {
            if repo.branch_exists(number.to_string().as_str()).is_err() {
                return Err(
                    git2::Error::from_str("branch already exists in remote. Please pull.").into(),
                );
            }
        }
        _ => unimplemented!(),
    }

    Ok(true)
}

pub(crate) fn slugify(string: &str) -> String {
    let separator_char = '-';
    let separator = separator_char.to_string();

    let string: String = unidecode(string.into())
        .to_lowercase()
        .trim_matches(separator_char)
        .replace(' ', &separator);

    let mut slug = Vec::with_capacity(string.len());
    let mut is_sep = true;

    for x in string.chars() {
        match x {
            'a'..='z' | '0'..='9' => {
                is_sep = false;
                slug.push(x as u8);
            }
            _ => {
                if !is_sep {
                    is_sep = true;
                    slug.push(separator_char as u8);
                }
            }
        }
    }

    if slug.last() == Some(&(separator_char as u8)) {
        slug.pop();
    }

    let s = String::from_utf8(slug).unwrap();
    s.trim_end_matches(separator_char).to_string()
}

// TODO: where does this belong
pub(crate) fn is_valid_file(path: &Path) -> bool {
    MARKUP_FORMAT_EXTENSIONS.contains_key(&path.extension().unwrap().to_str().unwrap())
}

#[cfg(test)]
mod tests {}
