use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

use chrono::{DateTime, Local};
use directories::UserDirs;
use serde::Serialize;
use tracing::debug;
use walkdir::{DirEntry, WalkDir};

use crate::cmd::design_decisions::is_valid_file;
use crate::files::{ensure_path, friendly_filename};
use crate::markup_format::MarkupFormat;
use crate::settings::{Config, SettingErrors, TilSettings, DEFAULT_TIL_DIR};
use crate::templates::{get_template, get_template_content, get_title};
use crate::templating::{TemplateContext, TemplateType, Templates, TilTemplateType};
use crate::{edit, CliResult, DoctaviousCliError};

#[derive(Clone, Debug, Serialize)]
struct TilEntry {
    topic: String,
    title: String,
    file_name: String,
    description: String,
    date: String,
}

// TODO: should probably be part of CLI docs.
/// Init TIL settings using the following rules
///
/// If `local` is true create settings in either `cwd` or the current directory
/// If `local` if false create global settings with `til.dir` as either `cwd` or default til directory
///
/// Global settings exist at the following locations
/// - on Linux: /home/<user>/.config/com.doctavious.cli/doctavious.toml
/// - on Windows: C:\Users\<user>\AppData\Roaming\com.doctavious.cli\doctavious.toml
/// - on macOS: /Users/<user>/Library/Application Support/com.doctavious.cli/doctavious.toml
pub fn init(cwd: Option<&Path>, format: MarkupFormat, local: bool) -> CliResult<PathBuf> {
    let cwd = cwd.and_then(|p| Some(p.to_path_buf()));
    let (mut config, til_dir) = if local {
        let path = cwd.unwrap_or(env::current_dir()?);
        (Config::from_path_or_default(&path), path)
    } else {
        let til_dir = cwd.unwrap_or(
            UserDirs::new()
                .expect("Could not get user directory")
                .home_dir()
                .join(DEFAULT_TIL_DIR),
        );
        (Config::get_global_or_default(), til_dir)
    };

    if config.settings.til_settings.is_some() {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::AlreadyInitialized("til".to_string()),
        ));
    }

    create_til_dir(&til_dir)?;

    config.settings.til_settings = Some(TilSettings {
        // dont need to include dir in a local config
        dir: if local {
            None
        } else {
            Some(til_dir.to_owned())
        },
        template_format: format,
    });

    config.save()?;

    Ok(til_dir)
}

fn create_til_dir(path: &Path) -> CliResult<()> {
    match fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            debug!("the directory {} already exists", path.to_string_lossy());
            Ok(())
        }
        Err(e) => {
            debug!(
                "Error occurred creating directory {}: {}",
                path.to_string_lossy(),
                e
            );
            Err(e.into())
        }
    }
}

// If `cwd` is provided but it doesnt have settings should we check if global settings exist?
// Should we allow people to customize where ToC write to?
/// Create new TIL post
///
/// If `cwd` is provided a create new TIL in that directory. Use settings from `cwd` if present, otherwise fallback to
/// global if present and default when not.
/// If `cwd` is not present see if current directory has settings, if present use, otherwise see if global
/// settings are present, if they are use them, otherwise default to current directory.
pub fn new(
    cwd: Option<&Path>,
    post: String,
    tags: Option<Vec<String>>,
    toc: bool,
) -> CliResult<PathBuf> {
    let config = get_config(cwd.as_deref())?;

    let Some((category, title)) = post.split_once("/") else {
        todo!()
    };

    let til_dir = if let Some(til_dir) = config.settings.get_til_dir() {
        til_dir
    } else {
        if let Some(cwd) = cwd {
            cwd.to_path_buf()
        } else {
            env::current_dir().expect("Unable to get current directory")
        }
    };

    let format = MarkupFormat::from_path(Path::new(title))
        .ok()
        .unwrap_or(config.settings.get_til_template_format(None));

    let path = Path::new(&til_dir)
        .join(category)
        .join(friendly_filename(title))
        .with_extension(format.extension());

    if path.is_file() {
        return Err(DoctaviousCliError::TilAlreadyExists);
    }

    ensure_path(&path)?;

    let template = get_template(
        &til_dir,
        TemplateType::Til(TilTemplateType::Post),
        &format.extension(),
    );

    let template_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("title", &friendly_title(title));
    context.insert("date", &Local::now().format("%Y-%m-%d").to_string());
    if let Some(tags) = tags {
        context.insert("tags", &tags.join(" ").to_string());
    }

    let rendered = Templates::one_off(template_content.as_str(), context, false)?;

    let edited = edit::edit(&rendered)?;
    fs::write(&path, edited)?;

    if toc {
        generate_toc(&til_dir, format)?;
    }

    Ok(path)
}

pub fn list(cwd: &Path) -> CliResult<Vec<PathBuf>> {
    let paths: Vec<_> = get_posts(cwd, None)
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(paths)
}

pub fn open(cwd: Option<&Path>, post: String) -> CliResult<PathBuf> {
    let config = get_config(cwd)?;

    // TODO: extract to fn
    let til_dir = if let Some(til_dir) = config.settings.get_til_dir() {
        til_dir
    } else {
        if let Some(cwd) = cwd {
            cwd.to_path_buf()
        } else {
            env::current_dir().expect("Unable to get current directory")
        }
    };

    let Some((topic, title)) = post.split_once("/") else {
        todo!()
    };

    let path = get_posts(&til_dir, Some(topic))
        .filter_map(|e| {
            if e.path().to_string_lossy().contains(title) {
                Some(e.path().to_path_buf())
            } else {
                None
            }
        })
        .next();

    if let Some(post_path) = path {
        let edited = edit::edit_path(&post_path)?;
        fs::write(&post_path, edited)?;
        Ok(post_path)
    } else {
        todo!()
    }
}

pub fn render() -> CliResult<()> {
    Ok(())
}

fn get_config(cwd: Option<&Path>) -> CliResult<Config> {
    Ok(if let Some(cwd) = cwd {
        Config::from_path_or_default(&cwd)
    } else {
        let local_config = Config::from_path_or_default(&env::current_dir()?);
        if local_config.is_default_settings {
            Config::get_global().unwrap_or_else(|_| local_config)
        } else {
            local_config
        }
    })

    // TODO: why does this cause tests to hang?
    // let path = cwd.and_then(|p| Some(p.to_path_buf())).unwrap_or(env::current_dir()?);
    // let local_config = Config::from_path_or_default(&path);
    // Ok(if local_config.is_default_settings {
    //     Config::get_global().unwrap_or_else(|_| local_config)
    // } else {
    //     local_config
    // })
}

fn get_posts(cwd: &Path, topic: Option<&str>) -> impl Iterator<Item = DirEntry> {
    let mut dir = cwd.to_path_buf();
    if let Some(topic) = topic {
        dir = dir.join(topic);
    }

    // TODO: should probably ignore hidden directories but that messes up tests which uses temp_dir
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| is_valid_file(&e.path()))
}

pub fn generate_toc(dir: &Path, format: MarkupFormat) -> CliResult<()> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in get_posts(&dir, None) {
        if let Some(til) = file_to_til_entry(dir, entry) {
            if let Some(tils) = all_tils.get_mut(&til.topic) {
                tils.push(til);
            } else {
                all_tils.insert(til.topic.clone(), vec![til]);
            }
        }
    }

    let mut til_count = 0;
    for topic_tils in all_tils.values() {
        til_count += topic_tils.len();
    }

    let template = get_template_content(
        dir,
        TemplateType::Til(TilTemplateType::ReadMe),
        format.extension(),
    );

    let mut context = TemplateContext::new();
    context.insert("categories_count", &all_tils.keys().len());
    context.insert("til_count", &til_count);
    context.insert("tils", &all_tils);

    let rendered = Templates::one_off(template.as_str(), context, false)?;
    let p = dir.join("README").with_extension(format.extension());
    fs::write(p, &rendered)?;

    Ok(())
}

fn file_to_til_entry(root: &Path, entry: DirEntry) -> Option<TilEntry> {
    let entry_path = entry.path();
    let parent = entry.path().parent()?;
    // skip files that are under til dir
    if root == parent {
        debug!("Skipping {:?}. Not in a topic", entry_path);
        return None;
    }

    let topic = parent.file_name();
    if topic.is_none() {
        debug!("Skipping {:?}. Could not get topic", entry_path);
        return None;
    }

    let title = entry_path.file_stem();
    if title.is_none() {
        debug!("Skipping {:?}. Could not get title", entry_path);
        return None;
    }

    let markup_format = MarkupFormat::from_path(entry_path);
    if markup_format.is_err() {
        debug!("Skipping {:?}. Could not get markup format", entry_path);
        return None;
    }

    let file = File::open(entry_path);
    if file.is_err() {
        debug!("Skipping {:?}. Could not open file", entry_path);
        return None;
    }

    let buffer = BufReader::new(file.ok()?);
    let description = get_title(buffer, markup_format.ok()?);
    let file_name = entry.path().file_name();
    if file_name.is_none() {
        debug!("Skipping {:?}. Could not get file_name", entry_path);
        return None;
    }

    let created = entry.metadata().ok().and_then(|m| m.created().ok());
    if created.is_none() {
        debug!("Skipping {:?}. Could not get created date", entry_path);
        return None;
    }

    Some(TilEntry {
        topic: topic?.to_string_lossy().to_string(),
        title: title?.to_string_lossy().to_string(),
        description,
        file_name: file_name?.to_string_lossy().to_string(),
        date: DateTime::<Local>::from(created?)
            .format("%Y-%m-%d")
            .to_string(),
    })
}

// TODO: where to put this
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn friendly_title(s: &str) -> String {
    s.split(&['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}


#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use directories::BaseDirs;
    use tempfile::TempDir;

    use crate::cmd::til::{init, list, new, open};
    use crate::files::get_all_files;
    use crate::markup_format::MarkupFormat;
    use crate::settings::Config;

    #[test]
    fn should_successfully_init_local() {
        let dir = TempDir::new().unwrap();
        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let path = init(Some(dir.path()), MarkupFormat::Markdown, true)
                    .expect("Should be able to init TIL");

                assert!(path.is_dir());
                let files = get_all_files(dir.path());
                assert_eq!(1, files.len());

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(&files[0]).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    // TODO: setup for CI flag
    #[test]
    // #[cfg(ci)]
    fn should_successfully_init_global() {
        let dir = TempDir::new().unwrap();
        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let path = init(Some(dir.path()), MarkupFormat::Markdown, false)
                    .expect("Should be able to init TIL");

                assert!(path.is_dir());
                let files = get_all_files(dir.path());

                // should not have any files in the TIL dir after init for global settings
                assert!(files.is_empty());

                let config = Config::get_global().unwrap();
                assert!(!config.is_default_settings);

                let expected_config_path = BaseDirs::new()
                    .unwrap()
                    .config_dir()
                    .join("com.doctavious.cli/doctavious.toml")
                    .to_string_lossy()
                    .to_string();

                assert!(config
                    .path
                    .to_string_lossy()
                    .ends_with(expected_config_path.as_str()));

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_yaml_snapshot!(&config);
                    insta::assert_snapshot!(fs::read_to_string(&config.path).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_successfully_create_new_post() {
        let dir = TempDir::new().unwrap();
        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let path = new(
                    Some(dir.path()),
                    "rust/testing".to_string(),
                    None,
                    false,
                )
                .expect("Should be able to create new post");

                insta::with_settings!({filters => vec![
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(path).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_edit_on_new_post() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/fake-editor")))],
            || {
                let path = new(
                    Some(dir.path()),
                    "rust/testing".to_string(),
                    None,
                    false,
                )
                .expect("Should be able to create new post");

                let content = fs::read_to_string(&path).unwrap();
                assert!(content.starts_with("EDITOR"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_successfully_create_toc() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let path = new(
                    Some(dir.path()),
                    "rust/testing".to_string(),
                    None,
                    true,
                )
                .expect("Should be able to create new post and ToC");

                insta::with_settings!({filters => vec![
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(dir.path().join("README.md")).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_list() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                new(
                    Some(dir.path()),
                    "rust/testing".to_string(),
                    None,
                    false,
                )
                .expect("Should be able to create new post");

                new(
                    Some(dir.path()),
                    "baz/foo".to_string(),
                    None,
                    false,
                )
                .expect("Should be able to create new post");

                let all_posts = list(dir.path()).unwrap();

                assert_eq!(2, all_posts.len());
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_open() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [("VISUAL", Some(Path::new("./tests/fixtures/fake-visual")))],
            || {
                new(
                    Some(dir.path()),
                    "rust/testing".to_string(),
                    None,
                    false,
                )
                .expect("Should be able to create new post");

                let path = open(Some(dir.path()), "rust/testing".to_string()).unwrap();
                assert!(fs::read_to_string(&path).unwrap().contains("VISUAL"));
            },
        );

        dir.close().unwrap();
    }
}
