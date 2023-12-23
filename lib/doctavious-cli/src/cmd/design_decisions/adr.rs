use std::fmt::Display;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::{Path, PathBuf};

use chrono::Utc;
use dotavious::{Dot, Edge, GraphBuilder, Node};
use git2::Repository;
use glob::glob;
use regex::RegexBuilder;
use walkdir::WalkDir;

use crate::cmd::design_decisions::{build_path, format_number, is_valid_file, reserve_number};
use crate::file_structure::FileStructure;
use crate::files::ensure_path;
use crate::markup_format::MarkupFormat;
use crate::settings::{init_dir, load_settings, persist_settings, AdrSettings, DEFAULT_ADR_DIR};
use crate::templates::{get_description, get_template};
use crate::templating::{AdrTemplateType, TemplateContext, TemplateType, Templates};
use crate::{edit, git, CliResult, DoctaviousCliError};

// TODO(Sean): might not be a great idea to include setting related stuff here in the lib
// as it might make it more difficult to use in various other scenarios. Fine for now but
// worth considering how we might want to structure to remove doctavious settings

/// Initialises the directory of architecture decision records:
/// * creates a subdirectory of the current working directory
/// * creates the first ADR in that subdirectory, recording the decision to record architectural decisions with ADRs.
pub(crate) fn init(
    cwd: &Path,
    path: Option<PathBuf>,
    structure: FileStructure,
    extension: Option<MarkupFormat>,
) -> CliResult<PathBuf> {
    // let mut settings = load_settings().unwrap_or_else(|_| Default::default());
    let mut settings = load_settings()?.into_owned();
    let path = path.unwrap_or_else(|| PathBuf::from(DEFAULT_ADR_DIR));
    let adr_dir = cwd.join(path);
    if adr_dir.exists() {
        return Err(DoctaviousCliError::DesignDocDirectoryAlreadyExists);
    }

    let directory_string = adr_dir.to_string_lossy().to_string();
    settings.adr_settings = Some(AdrSettings {
        dir: Some(directory_string),
        structure: Some(structure),
        template_extension: extension,
    });

    persist_settings(&settings)?;
    init_dir(&adr_dir)?;

    let adr_extension = settings.get_adr_template_extension(extension);
    return new(
        Some(adr_dir.as_path()),
        Some(1),
        "Record Architecture Decisions",
        AdrTemplateType::Init,
        adr_extension,
        None,
        None,
    );
}

/// Create a new ADR
///
/// This does not require `init` to be called prior as it will use appropriate defaults
pub(crate) fn new(
    cwd: Option<&Path>,
    number: Option<i32>,
    title: &str,
    template_type: AdrTemplateType,
    extension: MarkupFormat,
    supersedes: Option<Vec<String>>,
    links: Option<Vec<String>>,
) -> CliResult<PathBuf> {
    let settings = load_settings()?.into_owned();
    let dir = if let Some(cwd) = cwd {
        cwd
    } else {
        Path::new(settings.get_adr_dir())
    };

    let template = get_template(
        dir,
        TemplateType::Adr(template_type),
        &extension.extension(),
    );
    let reserve_number = reserve_number(dir, number, settings.get_adr_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let adr_path = build_path(
        dir,
        title,
        &formatted_reserved_number,
        extension,
        settings.get_adr_structure(),
    );

    ensure_path(&adr_path)?;

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

    // TODO:
    if let Some(targets) = supersedes {
        for target in targets {
            link(target.as_str(), "superseded by", rendered.as_str())?;
            // TODO: Do we care if its "Accepted"?
            remove_status(target.as_str(), "Accepted")?;
            link(rendered.as_str(), "supersedes", target.as_str())?;
        }
    }

    // TODO
    if let Some(links) = links {
        // links look like: "5:Amends:Amended by"
        for l in links {
            let parts = l.split(":").collect::<Vec<&str>>();
            if parts.len() != 3 {
                // error / warn / etc...
            }
            link(rendered.as_str(), parts[1], parts[0])?;
            link(parts[0], parts[2], rendered.as_str())?;
        }
    }

    let edited = edit::edit(&rendered)?;
    fs::write(&adr_path, edited)?;
    Ok(adr_path)
}

// TODO: cwd should be optional and default
// TODO: format should be optional? Try and determine from settings otherwise either default or look for both
pub fn list(cwd: PathBuf, format: MarkupFormat) -> CliResult<Vec<PathBuf>> {
    // this does a recursive search rather than a read_dir because we supported nested directory structures
    let mut paths = Vec::new();
    for entry in glob(format!("{}/**/*.{}", cwd.to_string_lossy(), format.extension()).as_str())? {
        if let Ok(entry) = entry {
            paths.push(entry);
        }
    }

    paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(paths)
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
pub(crate) fn reserve(
    cwd: Option<&Path>,
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> CliResult<()> {
    let settings = load_settings()?;
    let dir = if let Some(cwd) = cwd {
        cwd
    } else {
        Path::new(settings.get_adr_dir())
    };
    let reserve_number = reserve_number(dir, number, settings.get_adr_structure())?;

    // TODO: support more than current directory
    let repo = Repository::open(".")?;
    if git::branch_exists(&repo, reserve_number).is_err() {
        // TODO: use a different error than git2
        return Err(git2::Error::from_str("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str())?;

    let new_adr = new(
        Some(dir),
        number,
        title.as_str(),
        AdrTemplateType::Template,
        extension,
        None,
        None,
    )?;

    let message = format!("{}: Adding placeholder for ADR {}", reserve_number, title);
    git::add_and_commit(&repo, new_adr.as_path(), message.as_str())?;
    git::push(&repo)?;

    Ok(())
}

/// Creates a link between two ADRs, from SOURCE to TARGET new
/// SOURCE and TARGET are both a reference (number or partial filename) to an ADR
/// LINK is the description of the link created in the SOURCE.
/// REVERSE-LINK is the description of the link created in the TARGET
pub(crate) fn link(source: &str, link: &str, target: &str) -> CliResult<()> {
    let target_file = get_file(target).ok_or(DoctaviousCliError::UnknownDesignDocument(
        target.to_string(),
    ))?;

    let f = fs::File::open(&source)?;
    let reader = BufReader::new(f);
    let mut in_status_section = false;
    let mut target_title = None;

    let mut new_lines = vec![];

    // TODO implement link
    // find "## Status"
    // then find next "##" header
    // insert link
    // EX: adr new -l "1:Amends:Amended by" -l "2:Clarifies:Clarified by" Third Record
    // ## Status
    //
    // Accepted
    //
    // Amends [1. First Record](0001-first-record.md)
    //
    // Clarifies [2. Second Record](0002-second-record.md)
    //
    // ## Context

    // TODO(Sean): while this logic is straight forward I might, some day, want to swap for
    // modifying an AST to make changes.
    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("# ") {
                target_title = Some(line[2..].to_string());
            }

            if line == "## Status" {
                in_status_section = true;
            } else if line.starts_with("##") {
                if in_status_section {
                    new_lines.push(format!(
                        "{link} [{}]({})",
                        target_title.clone().unwrap_or_default(), // TODO: not sure how to avoid the clone
                        target_file.to_string_lossy()
                    ));
                    new_lines.push(String::new());
                }
                in_status_section = false;
            }

            new_lines.push(line);
        }
    }

    Ok(())
}

pub(crate) fn remove_status(file: &str, current_status: &str) -> CliResult<()> {
    let f = fs::File::open(file)?;
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

    Ok(())
}

pub(crate) fn generate_csv() {}

// TODO: not a fan of the list ToC for ADRs and RFDs
// TODO: pass in header
pub(crate) fn generate_toc(
    dir: &Path,
    extension: MarkupFormat,
    intro: Option<&str>,
    outro: Option<&str>,
    link_prefix: Option<&str>,
) -> String {
    let leading_char = extension.leading_header_character();
    let mut content = String::new();
    content.push_str(&format!(
        "{} {}\n\n",
        leading_char, "Architecture Decision Records"
    ));

    if intro.is_some() {
        content.push_str(&intro.unwrap());
        content.push_str("\n\n");
    }

    match fs::metadata(&dir) {
        Ok(_) => {
            let link_prefix = link_prefix.unwrap_or("");
            for entry in WalkDir::new(&dir)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|f| is_valid_file(&f.path()))
            {
                let file = match fs::File::open(&entry.path()) {
                    Ok(file) => file,
                    Err(_) => panic!("Unable to read file {:?}", entry.path()),
                };

                println!("{}", fs::read_to_string(&entry.path()).unwrap());

                let buffer = BufReader::new(file);
                let description = get_description(buffer, extension);
                // let file_name = entry.path().to_string_lossy().to_string();
                content.push_str(&format!(
                    "* [{}]({}{})\n",
                    description,
                    link_prefix,
                    entry.path().display()
                ));
            }
            content.push('\n');
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("the {} directory should exist", dir.to_string_lossy())
            }
            _ => eprintln!("Error occurred: {:?}", e),
        },
    }

    if outro.is_some() {
        content.push_str(&outro.unwrap());
    }

    content
}

pub(crate) fn graph_adrs() {
    let graph = GraphBuilder::new_named_directed("example")
        .add_node(Node::new("N0"))
        .add_node(Node::new("N1"))
        .add_edge(Edge::new("N0", "N1"))
        .build()
        .unwrap();

    let dot = Dot { graph };
}

fn get_file(target: &str) -> Option<PathBuf> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(target)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        if entry.file_name().to_string_lossy().contains(target) {
            paths.push(entry.path().to_path_buf());
        }
    }

    if paths.is_empty() {
        None
    } else {
        paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
        Some(paths.remove(0))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::TempDir;

    use crate::cmd::design_decisions::adr::{generate_toc, init, list, new};
    use crate::file_structure::FileStructure;
    use crate::markup_format::MarkupFormat;
    use crate::settings::DOCTAVIOUS_ENV_SETTINGS_PATH;
    use crate::templating::AdrTemplateType;

    // avoid-octal-numbers.expected

    #[test]
    fn create_first_record() {
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
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .expect("Should be able to create first new record");

                assert!(path
                    .to_string_lossy()
                    .ends_with("/0001-the-first-decision.md"));
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
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],
            || {
                new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    Some(dir.path()),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let third = new(
                    Some(dir.path()),
                    None,
                    "The Third Decision",
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                assert!(second
                    .to_string_lossy()
                    .ends_with("/0002-the-second-decision.md"));

                assert!(third
                    .to_string_lossy()
                    .ends_with("/0003-the-third-decision.md"));
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_edit_adr_on_create() {
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
                    AdrTemplateType::Template,
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
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("VISUAL", Some(Path::new("./tests/fixtures/fake-visual"))),
            ],
            || {
                let path = new(
                    Some(dir.path()),
                    None,
                    "The First Decision",
                    AdrTemplateType::Template,
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

    // generate contents
    #[test]
    fn should_generate_toc() {
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
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    Some(dir.path()),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let toc = generate_toc(dir.path(), MarkupFormat::Markdown, None, None, None);
            },
        );

        dir.close().unwrap();
    }

    #[test]
    fn should_generate_toc_with_header_footer() {
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
                    AdrTemplateType::Template,
                    MarkupFormat::Markdown,
                    None,
                    None,
                )
                .unwrap();

                let second = new(
                    Some(dir.path()),
                    None,
                    "The Second Decision",
                    AdrTemplateType::Template,
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
                );
            },
        );

        dir.close().unwrap();
    }

    // generate contents with prefix

    // generate graph

    // init ADR repository

    // init alternative ADR directory

    // linking new records

    // linking

    // list
    #[test]
    fn should_list() {
        // list(PathBuf::new(), MarkupFormat::Asciidoc).unwrap();
    }

    // must provide a title when creating new ADR

    // project specific template

    // search for ADR directory

    // search for custom ADR directory

    // supersede existing ADR

    // supersede multiple ADRs

    #[test]
    fn init_should_create_adr_directory_and_add_first_adr() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],|| {
            init(
                dir.path(),
                None,
                FileStructure::default(),
                Some(MarkupFormat::default()),
            )
            .expect("should init adr");
        });

        dir.close().unwrap();
    }

    #[test]
    fn init_with_custom_directory() {
        let dir = TempDir::new().unwrap();

        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],|| {
            let adr_path = init(
                dir.path(),
                Some(PathBuf::from("test/adrs")),
                FileStructure::default(),
                Some(MarkupFormat::default()),
            )
            .expect("should init adr");

            let trimmed_adr_path =
                &adr_path.to_string_lossy()[dir.path().to_string_lossy().len()..];

            assert!(trimmed_adr_path.starts_with("/test/adrs"));
        });

        dir.close().unwrap();
    }

    #[test]
    fn init_should_fail_on_non_empty_directory() {
        let dir = TempDir::new().unwrap();
        temp_env::with_vars(
            [
                (DOCTAVIOUS_ENV_SETTINGS_PATH, Some(dir.path())),
                ("EDITOR", Some(Path::new("./tests/fixtures/fake-editor"))),
            ],|| {
            init(
                dir.path(),
                None,
                FileStructure::default(),
                Some(MarkupFormat::default()),
            )
            .expect("should init adr");

            let adr_dir = init(
                dir.path(),
                None,
                FileStructure::default(),
                Some(MarkupFormat::default()),
            );

            assert!(adr_dir.is_err());
        });
        dir.close().unwrap();
    }

    // init options

    // init override existing

    // new w/o init

    // new with init
}
