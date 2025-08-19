use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use chrono::Local;
use doctavious_templating::{TemplateContext, Templates};
use markup::MarkupFormat;
use scm::drivers::{Scm, ScmRepository};
use serde::Serialize;

use crate::cmd::design_decisions;
use crate::cmd::design_decisions::{
    DesignDecisionErrors, build_path, can_reserve, format_number, reserve_number,
};
use crate::edit;
use crate::errors::{CliResult, DoctaviousCliError};
use crate::file_structure::FileStructure;
use crate::files::ensure_path;
use crate::settings::{
    DEFAULT_RFD_DIR, DEFAULT_RFD_RECORD_TEMPLATE_PATH, DEFAULT_RFD_TOC_TEMPLATE_PATH, RFDSettings,
    init_dir, load_settings, persist_settings,
};
use crate::templates::{get_template, get_title};
use crate::templating::{RfdTemplateType, TemplateType};

// RFD parsing: https://github.com/oxidecomputer/cio/blob/master/parse-rfd/src/lib.rs
// https://github.com/oxidecomputer/cio/tree/master/parse-rfd/parser

pub fn init(
    cwd: &Path,
    path: Option<PathBuf>,
    structure: FileStructure,
    format: MarkupFormat,
) -> CliResult<PathBuf> {
    let mut settings = load_settings(cwd)?;
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
        structure,
        template_format: format,
    };
    settings.rfd_settings = Some(rfd_settings);

    persist_settings(cwd, &settings)?;
    init_dir(&dir)?;

    // TODO: fix
    // https://github.com/gravitational/teleport/blob/master/rfd/0000-rfds.md
    new(cwd, Some(1), "Use RFDs ...", Some(format))
}

pub fn new(
    cwd: &Path,
    number: Option<u32>,
    title: &str,
    format: Option<MarkupFormat>,
) -> CliResult<PathBuf> {
    let settings = load_settings(cwd)?;
    let dir = get_rfd_dir(cwd, true)?;
    let format = settings.get_rfd_template_format(format);
    let template = get_template(
        &dir,
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
    context.insert("date", &Local::now().format("%Y-%m-%d").to_string());

    let rendered = Templates::one_off(starting_content.as_str(), &context, false)?;

    let edited = edit::edit(&rendered)?;
    fs::write(&output_path, edited)?;

    Ok(output_path)
}

// https://oxide.computer/blog/rfd-1-requests-for-discussion
// https://oxide.computer/blog/a-tool-for-discussion
pub fn reserve(
    cwd: &Path,
    number: Option<u32>,
    title: String,
    format: Option<MarkupFormat>,
) -> CliResult<()> {
    let settings = load_settings(cwd)?;
    let dir = get_rfd_dir(cwd, false)?;
    let format = settings.get_rfd_template_format(format);
    let reserve_number = reserve_number(&dir, number, settings.get_rfd_structure())?;

    let scm = Scm::get(cwd)?;
    can_reserve(&scm, reserve_number)?;

    let new_rfd = new(cwd, number, title.as_str(), Some(format))?;

    reserve_scm(&scm, reserve_number, &new_rfd, title)?;

    Ok(())
}

fn reserve_scm(repo: &Scm, number: u32, adr_path: &Path, title: String) -> CliResult<()> {
    match repo.scm() {
        scm::GIT => {
            repo.checkout(number.to_string().as_str())?;
            repo.write(
                adr_path,
                format!("{}: Adding placeholder for RFD {}", number, title).as_str(),
                None,
            )?;
        }
        _ => unimplemented!(),
    }

    Ok(())
}

pub fn list(cwd: &Path, format: MarkupFormat) -> CliResult<Vec<PathBuf>> {
    let dir = get_rfd_dir(cwd, false)?;
    Ok(design_decisions::list(&dir, format)?)
}

pub(crate) fn generate_csv() {}

pub(crate) fn generate_toc(
    cwd: &Path,
    format: MarkupFormat,
    intro: Option<&str>,
    outro: Option<&str>,
    link_prefix: Option<&str>,
) -> CliResult<String> {
    let dir = get_rfd_dir(cwd, false)?;

    #[derive(Clone, Debug, Serialize)]
    struct TocEntry {
        description: String,
        file_path: String,
    }

    let mut toc_entry = Vec::new();
    for p in list(&dir, format)? {
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
        &dir,
        TemplateType::Rfd(RfdTemplateType::ToC),
        &format.extension(),
    );

    let starting_content = fs::read_to_string(&template)?;

    Ok(Templates::one_off(
        starting_content.as_str(),
        &context,
        false,
    )?)
}

// TODO: option for global template
pub(crate) fn add_custom_template(
    cwd: &Path,
    template: RfdTemplateType,
    format: MarkupFormat,
    content: &str,
) -> CliResult<()> {
    let dir = get_rfd_dir(cwd, false)?;

    let template_path = match template {
        RfdTemplateType::Record => DEFAULT_RFD_RECORD_TEMPLATE_PATH,
        RfdTemplateType::ToC => DEFAULT_RFD_TOC_TEMPLATE_PATH,
    };

    let path = dir.join(template_path).with_extension(format.extension());
    fs::create_dir_all(&path.parent().expect("RFD template dir should have parent"))?;

    fs::write(&path, content)?;

    Ok(())
}

fn get_rfd_dir(cwd: &Path, create_if_missing: bool) -> CliResult<PathBuf> {
    let settings = load_settings(cwd)?;
    let path = cwd.join(settings.get_rfd_dir());

    if !path.is_dir() {
        if create_if_missing {
            fs::create_dir_all(&path)?;
        } else {
            return Err(DoctaviousCliError::DesignDecisionErrors(
                DesignDecisionErrors::DesignDocDirectoryInvalid,
            ));
        }
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use markup::MarkupFormat;
    use tempfile::TempDir;

    use crate::cmd::design_decisions::rfd::{add_custom_template, init, list, new};
    use crate::file_structure::FileStructure;
    use crate::templating::RfdTemplateType;

    #[test]
    fn create_first_record() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let path = new(dir.path(), None, "The First Decision", None)
                    .expect("Should be able to create first new record");

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
    fn create_multiple_records() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let first = new(dir.path(), None, "The First Decision", None).unwrap();

                let second = new(dir.path(), None, "The Second Decision", None).unwrap();

                let third = new(dir.path(), None, "The Third Decision", None).unwrap();

                insta::with_settings!({filters => vec![
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
            [("EDITOR", Some(Path::new("./tests/fixtures/fake-editor")))],
            || {
                let path = new(dir.path(), None, "The First Decision", None)
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
            [("VISUAL", Some(Path::new("./tests/fixtures/fake-visual")))],
            || {
                let path = new(dir.path(), None, "The First Decision", None)
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
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                let first = new(dir.path(), None, "The First Decision", None).unwrap();

                let second = new(dir.path(), None, "The Second Decision", None).unwrap();

                let rfds = list(dir.path(), MarkupFormat::Markdown).unwrap();

                assert_eq!(2, rfds.len());
                insta::with_settings!({filters => vec![
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
            [("EDITOR", Some(Path::new("./tests/fixtures/noop-editor")))],
            || {
                init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    MarkupFormat::default(),
                )
                .expect("should init adr");

                add_custom_template(
                    dir.path(),
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

                let custom_template =
                    new(dir.path(), None, "Custom Template Record", None).unwrap();

                insta::with_settings!({filters => vec![
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
            [("EDITOR", Some(Path::new("./tests/fixtures/fake-editor")))],
            || {
                let path = init(
                    dir.path(),
                    Some(PathBuf::from("test/rfds")),
                    FileStructure::default(),
                    MarkupFormat::default(),
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
            [("EDITOR", Some(Path::new("./tests/fixtures/fake-editor")))],
            || {
                init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    MarkupFormat::default(),
                )
                .expect("should init rfd");

                let dir = init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    MarkupFormat::default(),
                );

                assert!(dir.is_err());
            },
        );

        dir.close().unwrap();
    }
}
