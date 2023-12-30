use std::fmt::Display;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::Utc;
use git2::Repository;
use regex::RegexBuilder;
use serde::Serialize;

use crate::cmd::design_decisions;
use crate::cmd::design_decisions::{
    build_path, format_number, reserve_number, DesignDecisionErrors, LinkReference,
};
use crate::file_structure::FileStructure;
use crate::files::ensure_path;
use crate::markup_format::MarkupFormat;
use crate::settings::{
    init_dir, load_settings, persist_settings, AdrSettings, Settings, DEFAULT_ADR_DIR,
    DEFAULT_ADR_INIT_TEMPLATE_PATH, DEFAULT_ADR_RECORD_TEMPLATE_PATH,
    DEFAULT_ADR_TOC_TEMPLATE_PATH,
};
use crate::templates::{get_template, get_title};
use crate::templating::{AdrTemplateType, TemplateContext, TemplateType, Templates};
use crate::{edit, git, CliResult, DoctaviousCliError};

// TODO(Sean): might not be a great idea to include setting related stuff here in the lib
// as it might make it more difficult to use in various other scenarios. Fine for now but
// worth considering how we might want to structure to remove doctavious settings

/// Initialises the directory of architecture decision records:
/// * creates a subdirectory of the current working directory
/// * creates the first ADR in that subdirectory, recording the decision to record architectural decisions with ADRs.
pub fn init(
    cwd: &Path,
    path: Option<PathBuf>,
    structure: FileStructure,
    format: MarkupFormat,
) -> CliResult<PathBuf> {
    let mut settings: Settings = load_settings(cwd)?.into_owned();
    if settings.adr_settings.is_some() {
        return Err(DoctaviousCliError::DesignDecisionErrors(
            DesignDecisionErrors::DesignDocAlreadyInitialized,
        ));
    }

    let dir = cwd.join(path.unwrap_or_else(|| PathBuf::from(DEFAULT_ADR_DIR)));
    if dir.exists() {
        return Err(DoctaviousCliError::DesignDecisionErrors(
            DesignDecisionErrors::DesignDocDirectoryAlreadyExists,
        ));
    }

    let directory_string = dir.to_string_lossy().to_string();
    settings.adr_settings = Some(AdrSettings {
        dir: Some(directory_string),
        structure,
        template_format: format,
    });

    persist_settings(cwd, &settings)?;
    init_dir(&dir)?;

    return new(
        &dir,
        Some(1),
        "Record Architecture Decisions",
        AdrTemplateType::Init,
        format,
        None,
        None,
    );
}

/// Create a new ADR
///
/// This does not require `init` to be called prior as it will use appropriate defaults
pub fn new(
    cwd: &Path,
    number: Option<u32>,
    title: &str,
    template_type: AdrTemplateType,
    format: MarkupFormat,
    supersedes: Option<Vec<String>>,
    links: Option<Vec<String>>,
) -> CliResult<PathBuf> {
    let settings = load_settings(cwd)?.into_owned();
    let dir = get_adr_dir(cwd, true)?;

    let template = get_template(
        &dir,
        TemplateType::Adr(template_type),
        &format.extension(),
    );
    let reserve_number = reserve_number(&dir, number, settings.get_adr_structure())?;
    let formatted_reserved_number = format_number(&reserve_number);
    let output_path = build_path(
        &dir,
        title,
        &formatted_reserved_number,
        format,
        settings.get_adr_structure(),
    );

    ensure_path(&output_path)?;

    let starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    // TODO: allow date to be customized
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());

    let rendered = Templates::one_off(starting_content.as_str(), context, false)?;
    fs::write(&output_path, rendered.as_bytes())?;

    if let Some(targets) = supersedes {
        let dest_reference = LinkReference::Path(output_path.to_owned());
        for target in targets {
            let target_reference = LinkReference::from_str(target.as_str())?;
            // TODO: clean this up
            let target_path = target_reference.get_record(&dir).ok_or(
                DesignDecisionErrors::UnknownDesignDocument(target.to_string()),
            )?;
            add_link(
                dir.as_ref(),
                &target_reference,
                "Superseded by",
                &dest_reference,
            )?;
            remove_status(target_path.as_path(), "Accepted")?;
            add_link(
                dir.as_ref(),
                &dest_reference,
                "Supersedes",
                &target_reference,
            )?;
        }
    }

    if let Some(links) = links {
        let dest_reference = LinkReference::Path(output_path.to_owned());
        for l in links {
            let parts = l.split(":").collect::<Vec<&str>>();
            if parts.len() != 3 {
                // TODO: error / warn / etc...
            }

            let target_reference = LinkReference::from_str(parts[0])?;

            add_link(&dir, &dest_reference, parts[1], &target_reference)?;
            add_link(&dir, &target_reference, parts[2], &dest_reference)?;
        }
    }

    let edited = edit::edit_path(output_path.as_path())?;
    fs::write(&output_path, edited)?;
    Ok(output_path)
}

// TODO: format should be optional? Try and determine from settings otherwise either default or look for both
pub fn list(cwd: &Path, format: MarkupFormat) -> CliResult<Vec<PathBuf>> {
    let dir = get_adr_dir(cwd, false)?;

    Ok(design_decisions::list(&dir, format)?)
}

// implement ADR / RFD reserve command
// 1. get latest number
// 2. verify it doesnt exist
// git branch -rl *0042
// 3. checkout
// $ git checkout -b 0042
// 4. create the placeholder
// 5. Push your RFD branch remotely
// $ git add rfd/0042/README.md
// $ git commit -m '0042: Adding placeholder for RFD <Title>'
// $ git push origin 0042
// 6. Update README in main branch
// After your branch is pushed, the table in the README on the master branch will update
// automatically with the new RFD. If you ever change the name of the RFD in the future,
// the table will update as well. Whenever information about the state of the RFD changes,
// this updates the table as well. The single source of truth for information about the RFD comes
// from the RFD in the branch until it is merged.
// I think this would be implemented as a    git hook
pub fn reserve(
    cwd: &Path,
    number: Option<u32>,
    title: String,
    format: MarkupFormat,
) -> CliResult<()> {
    let settings = load_settings(cwd)?;
    let dir = get_adr_dir(cwd, false)?;
    let reserve_number = reserve_number(&dir, number, settings.get_adr_structure())?;

    let repo = Repository::open(&dir)?;
    if git::branch_exists(&repo, reserve_number).is_err() {
        // TODO: use a different error than git2
        return Err(git2::Error::from_str("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str())?;

    let new_adr = new(
        &dir,
        number,
        title.as_str(),
        AdrTemplateType::Record,
        format,
        None,
        None,
    )?;

    let message = format!("{}: Adding placeholder for ADR {}", reserve_number, title);
    git::add_and_commit(&repo, new_adr.as_path(), message.as_str())?;
    git::push(&repo)?;

    Ok(())
}

// TODO: This doc is better for the CLI
/// Creates a link between two ADRs, from SOURCE to TARGET new
/// SOURCE and TARGET are both a reference (number or partial filename) to an ADR
/// LINK is the description of the link created in the SOURCE.
/// REVERSE-LINK is the description of the link created in the TARGET
pub fn link(
    cwd: &Path,
    source: LinkReference,
    forward_link: &str,
    target: LinkReference,
    reverse_link: &str,
) -> CliResult<()> {
    let dir = get_adr_dir(cwd, false)?;
    add_link(&dir, &source, forward_link, &target)?;
    add_link(&dir, &target, reverse_link, &source)?;
    Ok(())
}

fn add_link(
    adr_dir: &Path,
    source: &LinkReference,
    link: &str,
    target: &LinkReference,
) -> CliResult<()> {
    let target_path = target
        .get_record(adr_dir)
        .ok_or(DesignDecisionErrors::UnknownDesignDocument(
            target.to_string(),
        ))?;

    let target_file = fs::File::open(&target_path)?;
    let target_title = get_title(
        BufReader::new(target_file),
        MarkupFormat::from_path(&target_path)?,
    );

    let source_path = source
        .get_record(adr_dir)
        .ok_or(DesignDecisionErrors::UnknownDesignDocument(
            source.to_string(),
        ))?;
    let source_content = fs::read_to_string(&source_path)?;

    let mut in_status_section = false;
    let mut new_lines = vec![];

    let formatted_link = if let Ok(stripped) = target_path.strip_prefix(adr_dir) {
        stripped.to_string_lossy()
    } else {
        target_path.to_string_lossy()
    };


    // TODO(Sean): while this logic is straight forward I might, some day, want to swap for
    // modifying an AST to make changes.
    for line in source_content.lines() {
        if line == "## Status" {
            in_status_section = true;
        } else if line.starts_with("##") {
            if in_status_section {
                new_lines.push(format!(
                    "{link} [{}]({})",
                    target_title.clone(), // TODO: not sure how to avoid the clone
                    formatted_link
                ));
                new_lines.push(String::new());
            }
            in_status_section = false;
        }

        new_lines.push(line.to_string());
    }

    fs::write(source_path, new_lines.join("\n"))?;
    Ok(())
}

fn remove_status(path: &Path, current_status: &str) -> CliResult<()> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    let mut in_status_section = false;
    let mut after_blank = false;
    let mut new_lines = vec![];

    // TODO: compile this?
    let regex = RegexBuilder::new(r"^\s*$").build()?;

    // TODO(Sean): while this logic is straight forward I might, some day, want to swap for
    // modifying an AST to make changes.
    for line in reader.lines() {
        if let Ok(line) = line {
            if line == "## Status" {
                in_status_section = true;
            } else if line.starts_with("##") {
                in_status_section = false;
            }

            // TODO: review logic. Originally from https://github.com/npryce/adr-tools/blob/master/src/_adr_remove_status
            if in_status_section && regex.is_match(&line) {
                if !after_blank {
                    new_lines.push(line);
                }
                after_blank = true;
                continue;
            }

            if in_status_section && line == current_status {
                continue;
            }

            if in_status_section && !regex.is_match(&line) {
                after_blank = false;
            }

            new_lines.push(line);
        }
    }

    fs::write(path, new_lines.join("\n"))?;
    Ok(())
}

pub fn generate_csv() {}

pub fn generate_toc(
    cwd: &Path,
    format: MarkupFormat,
    intro: Option<&str>,
    outro: Option<&str>,
    link_prefix: Option<&str>,
) -> CliResult<String> {
    let dir = get_adr_dir(cwd, false)?;

    #[derive(Clone, Debug, Serialize)]
    struct AdrEntry {
        description: String,
        file_path: String,
    }

    let mut adrs = Vec::new();
    for p in design_decisions::list(dir.as_ref(), format)? {
        let file = match fs::File::open(p.as_path()) {
            Ok(file) => file,
            Err(_) => panic!("Unable to read file {:?}", p),
        };

        let buffer = BufReader::new(file);
        let description = get_title(buffer, format);

        let file_path = p.to_string_lossy().trim_start_matches("./").to_string();

        adrs.push(AdrEntry {
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

    // let link_prefix = link_prefix
    //     .unwrap_or_default();

    context.insert("link_prefix", link_prefix.unwrap_or_default());
    context.insert("adrs", &adrs);

    let template = get_template(
        dir.as_ref(),
        TemplateType::Adr(AdrTemplateType::ToC),
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
    cwd: &Path,
    template: AdrTemplateType,
    format: MarkupFormat,
    content: &str,
) -> CliResult<()> {
    let dir = get_adr_dir(cwd, false)?;

    let template_path = match template {
        AdrTemplateType::Init => DEFAULT_ADR_INIT_TEMPLATE_PATH,
        AdrTemplateType::Record => DEFAULT_ADR_RECORD_TEMPLATE_PATH,
        AdrTemplateType::ToC => DEFAULT_ADR_TOC_TEMPLATE_PATH,
    };

    let path = dir.join(template_path).with_extension(format.extension());
    fs::create_dir_all(&path.parent().expect("ADR template dir should have parent"))?;

    fs::write(&path, content)?;

    Ok(())
}

fn get_adr_dir(cwd: &Path, create_if_missing: bool) -> CliResult<PathBuf> {
    let settings = load_settings(cwd)?;
    let path = cwd.join(settings.get_adr_dir());

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

    use tempfile::TempDir;

    use crate::cmd::design_decisions::adr::{
        add_custom_template, generate_toc, init, link, list, new,
    };
    use crate::cmd::design_decisions::LinkReference;
    use crate::file_structure::FileStructure;
    use crate::markup_format::MarkupFormat;
    use crate::settings::DOCTAVIOUS_ENV_SETTINGS_PATH;
    use crate::templating::AdrTemplateType;

    #[test]
    fn create_first_record() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let path = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
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
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let third = new(
                    dir.path(),
                    None,
                    "The Third Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
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
    fn should_edit_adr_on_create() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                let path = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .expect("Should be able to create first new record");

                let content = fs::read_to_string(&path).unwrap();
                assert!(content.starts_with("EDITOR"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_use_visual_edit_adr_on_create() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("VISUAL", Some(Path::new("./tests/fixtures/fake-visual"))),
            ],
            || {
                let path = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .expect("Should be able to create first new record");

                let content = fs::read_to_string(&path).unwrap();
                assert!(content.starts_with("VISUAL"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_generate_toc() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let toc = generate_toc(dir.path(), MarkupFormat::Markdown, None, None, None)
                    .unwrap();

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                ]}, {
                    insta::assert_snapshot!(toc);
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_generate_toc_with_header_footer() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let toc = generate_toc(
                    dir.path(),
                    MarkupFormat::Markdown,
                    Some(
                        r#"An intro.

Multiple paragraphs."#,
                    ),
                    Some("An outro."),
                    None,
                )
                .unwrap();

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                ]}, {
                    insta::assert_snapshot!(toc);
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_generate_toc_with_link_prefix() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let toc = generate_toc(
                    dir.path(),
                    MarkupFormat::Markdown,
                    None,
                    None,
                    Some("a-link-prefix"),
                )
                .unwrap();

                insta::with_settings!({filters => vec![
                    (dir.path().to_str().unwrap(), "[DIR]"),
                ]}, {
                    insta::assert_snapshot!(toc);
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_support_linking_adr() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "First Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "Second Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let third = new(
                    dir.path(),
                    None,
                    "Third Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                link(
                    dir.path(),
                    LinkReference::Number(3),
                    "Amends",
                    LinkReference::Number(1),
                    "Amended by",
                )
                .unwrap();

                link(
                    dir.path(),
                    LinkReference::Number(3),
                    "Clarifies",
                    LinkReference::Number(2),
                    "Clarified by",
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
    fn should_support_linking_when_creating_new_adr() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "First Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "Second Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let third = new(
                    dir.path(),
                    None,
                    "Third Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    Some(vec![
                        "1:Amends:Amended by".to_string(),
                        "2:Clarifies:Clarified by".to_string(),
                    ]),
                )
                .unwrap();

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
    fn should_support_superseding_adr() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "First Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "Second Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    Some(vec!["1".to_string()]),
                    None,
                )
                .unwrap();

                insta::with_settings!({filters => vec![
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(first).unwrap());
                    insta::assert_snapshot!(fs::read_to_string(second).unwrap());
                });
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_support_superseding_multiple_adr() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "First Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "Second Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let third = new(
                    dir.path(),
                    None,
                    "Third Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    Some(vec!["1".to_string(), "2".to_string()]),
                    None,
                )
                .unwrap();

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
    fn should_list() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let first = new(
                    dir.path(),
                    None,
                    "The First Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    dir.path(),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let adrs = list(dir.path(), MarkupFormat::Markdown).unwrap();

                assert_eq!(2, adrs.len());
                insta::with_settings!({filters => vec![
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(fs::read_to_string(&adrs[0]).unwrap());
                    insta::assert_snapshot!(fs::read_to_string(&adrs[1]).unwrap());
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
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
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
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    r#"# TITLE

Project specific template!

# Status

STATUS

# Info

ADR Number: {{ number }}

Date: {{ date }}
"#,
                )
                .unwrap();

                let custom_template = new(
                    dir.path(),
                    None,
                    "Custom Template Record",
                    AdrTemplateType::Record,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

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
    fn init_should_create_adr_directory_and_add_first_adr() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/noop-editor"))),
            ],
            || {
                let path = init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    MarkupFormat::default(),
                )
                .expect("should init adr");

                let content = fs::read_to_string(path).unwrap();

                insta::with_settings!({filters => vec![
                    (r"\d{4}-\d{2}-\d{2}", "[DATE]")
                ]}, {
                    insta::assert_snapshot!(content);
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
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                let adr_path = init(
                    dir.path(),
                    Some(PathBuf::from("test/adrs")),
                    FileStructure::default(),
                    MarkupFormat::default(),
                )
                .expect("should init adr");

                let trimmed_adr_path =
                    &adr_path.to_string_lossy()[dir.path().to_string_lossy().len()..];

                assert!(trimmed_adr_path.starts_with("/test/adrs"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn init_should_fail_on_non_empty_directory() {
        let dir = TempDir::new().unwrap();
        temp_env::with_vars(
            [
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    MarkupFormat::default(),
                )
                .expect("should init adr");

                let adr_dir = init(
                    dir.path(),
                    None,
                    FileStructure::default(),
                    MarkupFormat::default(),
                );

                assert!(adr_dir.is_err());
            },
        );
        dir.close().unwrap();
    }
}
