use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs};

use chrono::{DateTime, Local, Utc};
use directories::UserDirs;
use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

use crate::cmd::design_decisions::is_valid_file;
use crate::files::{ensure_path, friendly_filename};
use crate::markup_format::MarkupFormat;
use crate::settings::{init_dir, Config, TilSettings, DEFAULT_TIL_DIR};
use crate::templates::{get_template, get_template_content, get_title};
use crate::templating::{TemplateContext, TemplateType, Templates, TilTemplateType};
use crate::{edit, CliResult};

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
/// - on Linux: /home/<user>/.config/com.doctavious.cli/doctavious.toml␊
/// - on Windows: C:\Users\<user>\AppData\Roaming\com.doctavious.cli\doctavious.toml␊
/// - on macOS: /Users/<user>/Library/Application Support/com.doctavious.cli/doctavious.toml␊
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
        // TODO: return error
    }

    init_dir(&til_dir)?;

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
    title: String,
    category: String,
    tags: Option<Vec<String>>,
    file_name: Option<String>,
    format: Option<MarkupFormat>,
    toc: bool,
) -> CliResult<PathBuf> {
    let config = get_config(cwd.as_deref())?;

    // let settings = get_settings(&settings_file)?;
    let til_dir = if let Some(til_dir) = config.settings.get_til_dir() {
        til_dir
    } else {
        if let Some(cwd) = cwd {
            cwd.to_path_buf()
        } else {
            env::current_dir().expect("Unable to get current directory")
        }
    };

    let format = config.settings.get_til_template_format(format);

    // https://stackoverflow.com/questions/7406102/create-sane-safe-filename-from-any-unsafe-string
    // https://docs.rs/sanitize-filename/latest/sanitize_filename/
    // https://github.com/danielecook/TIL-Tool/ has the following
    // arg("fname", nargs = 1, help="Filename in the format topic/title (e.g. R/create_matrix")
    // TODO: convert to a better file name
    // spaces to hyphens
    // special characters?
    let file_name = if let Some(file_name) = file_name {
        file_name.trim().to_string()
    } else {
        friendly_filename(&title)
    };

    let path = Path::new(&til_dir)
        .join(category)
        .join(file_name)
        .with_extension(format.extension());

    if path.is_file() {
        // TODO: this should return the error
        eprintln!("File {} already exists", path.to_string_lossy());
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
    context.insert("title", &title);
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

pub fn open(cwd: Option<&Path>, post: String) -> CliResult<()> {
    let config = get_config(cwd)?;
    let til_dir = config
        .settings
        .get_til_dir()
        .unwrap_or(env::current_dir().expect("Unable to get current directory"));
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
        Ok(())
    } else {
        // Err(DoctaviousCliError::)
        Ok(())
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

// TODO: this should just build_mod the content and return and not write
pub(crate) fn generate_toc(dir: &Path, format: MarkupFormat) -> CliResult<()> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in get_posts(&dir, None) {
        // skip files that are under til dir
        if dir == entry.path().parent().unwrap() {
            continue;
        }

        // TODO: handle unwraps better
        let topic = entry
            .path()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        if !all_tils.contains_key(&topic) {
            // TODO: is there a way to avoid this clone?
            all_tils.insert(topic.clone(), Vec::new());
        }

        let markup_format =
            MarkupFormat::from_str(entry.path().extension().unwrap().to_str().unwrap()).unwrap();

        let file = File::open(&entry.path())?;
        let buffer = BufReader::new(file);
        // TODO: should this use extension to get title? Would allow for users to mix/match file types
        let description = get_title(buffer, markup_format);
        // let file_name = entry.file_name().to_string_lossy().to_string();
        // let entry_path = entry.path();
        // let file_name = if let Ok(stripped) = entry_path.strip_prefix(dir) {
        //     stripped.to_string_lossy().to_string()
        // } else {
        //     entry_path.to_string_lossy().to_string()
        // };
        let file_name = entry.path().file_name()
            .expect("Unable to get file name")
            .to_string_lossy().to_string();

        all_tils.get_mut(&topic).unwrap().push(TilEntry {
            topic,
            title: entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string(), // TODO: dont unwrap
            description,
            file_name,
            date: DateTime::<Local>::from(entry.metadata()?.created()?).format("%Y-%m-%d").to_string(),
        });
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

// TODO: where to put this
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use crate::cmd::til::{init, new};
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
                    "testing".to_string(),
                    "rust".to_string(),
                    None,
                    None,
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
                    "testing".to_string(),
                    "rust".to_string(),
                    None,
                    None,
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
                    "testing".to_string(),
                    "rust".to_string(),
                    None,
                    None,
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
}
