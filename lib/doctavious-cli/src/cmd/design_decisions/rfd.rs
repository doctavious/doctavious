use std::borrow::{Borrow, Cow};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use chrono::Utc;
use git2::Repository;
use serde::Serialize;

use crate::cmd::design_decisions;
use crate::cmd::design_decisions::{
    build_path, format_number, reserve_number, DesignDecisionErrors,
};
use crate::file_structure::FileStructure;
use crate::files::ensure_path;
use crate::markup_format::MarkupFormat;
use crate::settings::{
    init_dir, load_settings, persist_settings, RFDSettings, DEFAULT_RFD_DIR,
    DEFAULT_RFD_RECORD_TEMPLATE_PATH, DEFAULT_RFD_TOC_TEMPLATE_PATH,
};
use crate::templates::{get_template, get_title};
use crate::templating::{RfdTemplateType, TemplateContext, TemplateType, Templates};
use crate::{edit, git, CliResult, DoctaviousCliError};

// RFD parsing: https://github.com/oxidecomputer/cio/blob/master/parse-rfd/src/lib.rs
// https://github.com/oxidecomputer/cio/tree/master/parse-rfd/parser

pub(crate) fn init(
    cwd: &Path,
    path: Option<PathBuf>,
    structure: FileStructure,
    extension: Option<MarkupFormat>,
) -> CliResult<PathBuf> {
    let mut settings = load_settings()?.into_owned();
    let path = path.unwrap_or_else(|| PathBuf::from(DEFAULT_RFD_DIR));
    let dir = cwd.join(path);
    if dir.exists() {
        return Err(DoctaviousCliError::DesignDecisionErrors(
            DesignDecisionErrors::DesignDocDirectoryAlreadyExists,
        ));
    }

    let directory_string = dir.to_string_lossy().to_string();
    let rfd_settings = RFDSettings {
        dir: Some(directory_string),
        structure: Some(structure),
        template_format: extension,
    };
    settings.rfd_settings = Some(rfd_settings);

    persist_settings(&settings)?;
    init_dir(&dir)?;

    let rfd_extension = settings.get_rfd_template_extension(extension);

    // TODO: fix
    // https://github.com/gravitational/teleport/blob/master/rfd/0000-rfds.md
    new(None, Some(1), "Use RFDs ...", rfd_extension)
}

pub(crate) fn new(
    cwd: Option<&Path>,
    number: Option<u32>,
    title: &str,
    format: MarkupFormat,
) -> CliResult<PathBuf> {
    let settings = load_settings()?;
    let dir = get_rfd_dir(cwd)?;

    let template = get_template(
        Path::new(dir.as_ref()),
        TemplateType::Rfd(RfdTemplateType::Record),
        &format.extension(),
    );
    let reserve_number = reserve_number(&dir, number, settings.get_rfd_structure())?;
    let formatted_reserved_number = format_number(&reserve_number);
    let output_path = build_path(
        &dir,
        &title,
        &formatted_reserved_number,
        format,
        settings.get_rfd_structure(),
    );

    ensure_path(&output_path)?;

    let starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());

    let rendered = Templates::one_off(starting_content.as_str(), context, false)?;

    let edited = edit::edit(&rendered)?;
    fs::write(&output_path, edited)?;

    Ok(output_path)
}

// https://oxide.computer/blog/rfd-1-requests-for-discussion
// https://oxide.computer/blog/a-tool-for-discussion
pub(crate) fn reserve(
    cwd: Option<&Path>,
    number: Option<u32>,
    title: String,
    format: MarkupFormat,
) -> CliResult<()> {
    let settings = load_settings()?;
    let dir = get_rfd_dir(cwd)?;

    let reserve_number = reserve_number(
        Path::new(dir.as_ref()),
        number,
        settings.get_rfd_structure(),
    )?;

    let repo = Repository::open(dir.as_ref())?;
    if git::branch_exists(&repo, reserve_number).is_err() {
        // TODO: use a different error than git2
        return Err(git2::Error::from_str("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str())?;

    let new_rfd = new(None, number, title.as_str(), format)?;

    let message = format!("{}: Adding placeholder for RFD {}", reserve_number, title);
    git::add_and_commit(&repo, new_rfd.as_path(), message.as_str())?;
    git::push(&repo)?;

    Ok(())
}

pub fn list(cwd: Option<&Path>, format: MarkupFormat) -> CliResult<Vec<PathBuf>> {
    let dir = get_rfd_dir(cwd)?;

    Ok(design_decisions::list(dir.as_ref(), format)?)
}

pub(crate) fn generate_csv() {}

pub(crate) fn generate_toc(
    cwd: Option<&Path>,
    format: MarkupFormat,
    intro: Option<&str>,
    outro: Option<&str>,
    link_prefix: Option<&str>,
) -> CliResult<String> {
    let dir = get_rfd_dir(cwd)?;

    #[derive(Clone, Debug, Serialize)]
    struct TocEntry {
        description: String,
        file_path: String,
    }

    let mut toc_entry = Vec::new();
    for p in list(Some(dir.as_ref()), format)? {
        let file = match fs::File::open(p.as_path()) {
            Ok(file) => file,
            Err(_) => panic!("Unable to read file {:?}", p),
        };

        let buffer = BufReader::new(file);
        let description = get_title(buffer, format);

        let file_path = p.to_string_lossy().trim_start_matches("./").to_string();

        toc_entry.push(TocEntry {
            description,
            file_path,
        });
    }

    let mut context = TemplateContext::new();
    if let Some(intro) = intro {
        context.insert("intro", intro);
    }
    if let Some(outro) = outro {
        context.insert("outro", outro);
    }

    context.insert("link_prefix", link_prefix.unwrap_or_default());
    context.insert("entries", &toc_entry);

    let template = get_template(
        dir.as_ref(),
        TemplateType::Rfd(RfdTemplateType::ToC),
        &format.extension(),
    );

    let starting_content = fs::read_to_string(&template)?;

    Ok(Templates::one_off(
        starting_content.as_str(),
        context,
        false,
    )?)
}

// TODO: option for global template
pub(crate) fn add_custom_template(
    cwd: Option<&Path>,
    template: RfdTemplateType,
    format: MarkupFormat,
    content: &str,
) -> CliResult<()> {
    let dir = get_rfd_dir(cwd)?;

    let template_path = match template {
        RfdTemplateType::Record => DEFAULT_RFD_RECORD_TEMPLATE_PATH,
        RfdTemplateType::ToC => DEFAULT_RFD_TOC_TEMPLATE_PATH,
    };

    let path = dir.join(template_path).with_extension(format.extension());
    fs::create_dir_all(&path.parent().expect("RFD template dir should have parent"))?;

    fs::write(&path, content)?;

    Ok(())
}

fn get_rfd_dir(cwd: Option<&Path>) -> CliResult<Cow<Path>> {
    let settings = load_settings()?.into_owned();
    if let Some(cwd) = cwd {
        if !cwd.is_dir() {
            return Err(DoctaviousCliError::DesignDecisionErrors(
                DesignDecisionErrors::DesignDocDirectoryInvalid,
            ));
        }
        Ok(Cow::Borrowed(cwd))
    } else {
        Ok(Cow::Owned(PathBuf::from(settings.get_rfd_dir())))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::cmd::design_decisions::rfd::{add_custom_template, init, list, new};
    use crate::file_structure::FileStructure;
    use crate::markup_format::MarkupFormat;
    use crate::settings::DOCTAVIOUS_ENV_SETTINGS_PATH;
    use crate::templating::RfdTemplateType;

    #[test]
    fn create_first_record() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let path = new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    MarkupFormat::Markdown,
                )
                .expect("Should be able to create first new record");

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(path).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn create_multiple_records() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    MarkupFormat::Markdown,
                )
                .unwrap();

                let second = new(
                    Some(dir.path()),
                    None,
                    "The Second Decision",
                    MarkupFormat::Markdown,
                )
                .unwrap();

                let third = new(
                    Some(dir.path()),
                    None,
                    "The Third Decision",
                    MarkupFormat::Markdown,
                )
                .unwrap();

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(first).unwrap());
                    insta::assert_snapshot!(fs::read_to_string(second).unwrap());
                    insta::assert_snapshot!(fs::read_to_string(third).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_edit_on_create() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                let path = new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    MarkupFormat::Markdown,
                )
                .expect("Should be able to create first new record");

                let content = fs::read_to_string(&path).unwrap();
                assert!(content.starts_with("EDITOR"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_use_visual_edit_on_create() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("VISUAL", Some(Path::new("./tests/fixtures/fake-visual"))),
            ],
            || {
                let path = new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    MarkupFormat::Markdown,
                )
                .expect("Should be able to create first new record");

                let content = fs::read_to_string(&path).unwrap();
                assert!(content.starts_with("VISUAL"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_list() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    MarkupFormat::Markdown,
                )
                .unwrap();

                let second = new(
                    Some(dir.path()),
                    None,
                    "The Second Decision",
                    MarkupFormat::Markdown,
                )
                .unwrap();

                let rfds = list(Some(dir.path()), MarkupFormat::Markdown).unwrap();

                assert_eq!(2, rfds.len());
                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(&rfds[0]).unwrap());
                    insta::assert_snapshot!(fs::read_to_string(&rfds[1]).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_allow_custom_project_template() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    Some(MarkupFormat::default()),
                )
                .expect("should init adr");

                add_custom_template(
                    Some(dir.path()),
                    RfdTemplateType::Record,
                    MarkupFormat::Markdown,
                    r#"# TITLE

Project specific template!

# Status

STATUS

# Info

RFD Number: {{ number }}

Date: {{ date }}
"#,
                )
                .unwrap();

                let custom_template = new(
                    Some(dir.path()),
                    None,
                    "Custom Template Record",
                    MarkupFormat::Markdown,
                )
                .unwrap();

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(custom_template).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn init_with_custom_directory() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                let path = init(
                    dir.path(),
                    Some(PathBuf::from("test/rfds")),
                    FileStructure::default(),
                    Some(MarkupFormat::default()),
                )
                .expect("should init RFDs");

                let trimmed_path = &path.to_string_lossy()[dir.path().to_string_lossy().len()..];

                assert!(trimmed_path.starts_with("/test/rfds"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn init_should_fail_on_non_empty_directory() {
        let dir = TempDir::new().unwrap();
        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    Some(MarkupFormat::default()),
                )
                .expect("should init rfd");

                let dir = init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    Some(MarkupFormat::default()),
                );

                assert!(dir.is_err());
            },
        );

        dir.close().unwrap();
    }
}
