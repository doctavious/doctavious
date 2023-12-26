use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

use crate::files::friendly_filename;
use crate::markup_format::MarkupFormat;
use crate::settings::{init_dir, load_settings, persist_settings, TilSettings, DEFAULT_TIL_DIR};
use crate::templates::{get_description, get_template_content};
use crate::templating::{TemplateContext, TemplateType, Templates, TilTemplateType};
use crate::{edit, CliResult};

#[derive(Clone, Debug, Serialize)]
struct TilEntry {
    topic: String,
    title: String,
    file_name: String,
    description: String,
    date: DateTime<Utc>,
}

pub(crate) fn init_til(directory: Option<PathBuf>, extension: MarkupFormat) -> CliResult<()> {
    let mut settings = load_settings()?.into_owned();
    let dir = directory.unwrap_or_else(|| PathBuf::from(DEFAULT_TIL_DIR));
    let directory_string = dir.to_string_lossy().to_string();

    let til_settings = TilSettings {
        dir: Some(directory_string),
        template_extension: Some(extension),
    };
    settings.til_settings = Some(til_settings);

    persist_settings(&settings)?;
    init_dir(&dir)?;

    Ok(())
}

pub(crate) fn new_til(
    title: String,
    category: String,
    tags: Option<Vec<String>>,
    file_name: Option<String>,
    markup_format: MarkupFormat,
    readme: bool,
    dir: &Path,
) -> CliResult<()> {
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

    let path = Path::new(dir)
        .join(category)
        .join(file_name)
        .with_extension(markup_format.extension());

    if path.is_file() {
        // TODO: this should return the error
        eprintln!("File {} already exists", path.to_string_lossy());
    }

    let leading_char = markup_format.leading_header_character();

    let mut starting_content = format!("{} {}\n", leading_char, title);
    if tags.is_some() {
        starting_content.push_str("\ntags: ");
        starting_content.push_str(tags.unwrap().join(" ").as_str());
    }

    let edited = edit::edit(&starting_content)?;

    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, edited)?;

    if readme {
        build_til_readme(dir, markup_format.extension())?;
    }

    Ok(())
}

// TODO: this should just build_mod the content and return and not write
pub(crate) fn build_til_readme(dir: &Path, readme_extension: &str) -> CliResult<String> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
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
        let description = get_description(buffer, markup_format);
        // let file_name = entry.file_name().to_string_lossy().to_string();
        let file_name = entry.path().to_string_lossy().to_string();
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
            date: DateTime::from(entry.metadata()?.created()?),
        });
    }

    let mut til_count = 0;
    for topic_tils in all_tils.values() {
        til_count += topic_tils.len();
    }

    let template = get_template_content(
        dir,
        TemplateType::Til(TilTemplateType::ReadMe),
        readme_extension,
    );
    let mut context = TemplateContext::new();
    context.insert("categories_count", &all_tils.keys().len());
    context.insert("til_count", &til_count);
    context.insert("tils", &all_tils);

    let rendered = Templates::one_off(template.as_str(), context, false)?;
    Ok(rendered)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
