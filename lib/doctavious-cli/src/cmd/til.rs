use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::Serialize;
use walkdir::{DirEntry, WalkDir};

use crate::files::friendly_filename;
use crate::markup_format::MarkupFormat;
use crate::settings::{
    init_dir, load_settings, persist_settings, TilSettings, DEFAULT_TIL_DIR,
    DEFAULT_TIL_TEMPLATE_PATH,
};
use crate::templates::{get_template_content, TemplateContext, Templates};
use crate::{edit, CliResult};

#[derive(Clone, Debug, Serialize)]
struct TilEntry {
    topic: String,
    title: String,
    file_name: String,
    date: DateTime<Utc>,
}

pub(crate) fn init_til(directory: Option<String>, extension: MarkupFormat) -> CliResult<()> {
    let mut settings = load_settings().unwrap_or_else(|_| Default::default());

    let dir = match directory {
        None => DEFAULT_TIL_DIR,
        Some(ref d) => d,
    };

    let til_settings = TilSettings {
        dir: Some(dir.to_string()),
        template_extension: Some(extension),
    };
    settings.til_settings = Some(til_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    return Ok(());
}

pub(crate) fn new_til(
    title: String,
    category: String,
    tags: Option<Vec<String>>,
    file_name: Option<String>,
    markup_format: MarkupFormat,
    readme: bool,
    dir: &str,
) -> CliResult<()> {
    // https://stackoverflow.com/questions/7406102/create-sane-safe-filename-from-any-unsafe-string
    // https://docs.rs/sanitize-filename/latest/sanitize_filename/
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

    if path.exists() {
        // TODO: this should return the error
        eprintln!("File {} already exists", path.to_string_lossy());
    } else {
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
            build_til_readme(&dir, markup_format.extension())?;
        }
    }

    return Ok(());
}

// TODO: this should just build_mod the content and return and not write
pub(crate) fn build_til_readme(dir: &str, readme_extension: &str) -> CliResult<String> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        // skip files that are under til dir
        if Path::new(dir) == entry.path().parent().unwrap() {
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

        let file_name = entry
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let markup_format =
            MarkupFormat::from_str(entry.path().extension().unwrap().to_str().unwrap()).unwrap();
        let file = match File::open(&entry.path()) {
            Ok(file) => file,
            Err(_) => panic!("Unable to read title from {:?}", entry.path()),
        };

        let buffer = BufReader::new(file);
        // TODO: should this use extension to get title? Would allow for users to mix/match file types
        let title = title_string(buffer, markup_format);

        all_tils.get_mut(&topic).unwrap().push(TilEntry {
            topic,
            title,
            file_name,
            date: DateTime::from(entry.metadata()?.created()?),
        });
    }

    let mut til_count = 0;
    for topic_tils in all_tils.values() {
        til_count += topic_tils.len();
    }

    let template = get_template_content(&dir, readme_extension, DEFAULT_TIL_TEMPLATE_PATH);
    let mut context = TemplateContext::new();
    context.insert("categories_count", &all_tils.keys().len());
    context.insert("til_count", &til_count);
    context.insert("tils", &all_tils);

    let rendered = Templates::one_off(template.as_str(), &context, false)?;
    return Ok(rendered);
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub(crate) fn title_string<R>(rdr: R, markup_format: MarkupFormat) -> String
where
    R: BufRead,
{
    // TODO: swap this implementation for AST when ready
    let leading_char = markup_format.leading_header_character();
    for line in rdr.lines() {
        let line = line.unwrap();
        if line.starts_with(leading_char) {
            let last_hash = line
                .char_indices()
                .skip_while(|&(_, c)| c == leading_char)
                .next()
                .map_or(0, |(idx, _)| idx);

            // Trim the leading hashes and any whitespace
            return line[last_hash..].trim().to_string();
        }
    }

    // TODO: dont panic. default to filename if cant get title
    panic!("Unable to find title for file");
}
