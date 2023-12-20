use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use dotavious::{Dot, Edge, GraphBuilder, Node};
use git2::Repository;

use crate::cmd::design_decisions::{build_path, format_number, reserve_number};
use crate::file_structure::FileStructure;
use crate::files::ensure_path;
use crate::markup_format::MarkupFormat;
use crate::settings::{
    init_dir, load_settings, persist_settings, RFDSettings, DEFAULT_RFD_DIR,
    DEFAULT_RFD_TEMPLATE_PATH, SETTINGS,
};
use crate::templates::{get_template, TemplateContext, Templates};
use crate::{edit, git, CliResult};

pub(crate) fn init_rfd(
    directory: Option<String>,
    structure: FileStructure,
    extension: MarkupFormat,
) -> CliResult<PathBuf> {
    let mut settings = load_settings().unwrap_or_else(|_| Default::default());

    let dir = match directory {
        None => DEFAULT_RFD_DIR,
        Some(ref d) => d,
    };

    let rfd_settings = RFDSettings {
        dir: Some(dir.to_string()),
        structure: Some(structure),
        template_extension: Some(extension),
    };
    settings.rfd_settings = Some(rfd_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    // TODO: fix
    return new_rfd(Some(1), "Use RFDs ...".to_string(), extension);
}

pub(crate) fn new_rfd(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> CliResult<PathBuf> {
    let dir = SETTINGS.get_rfd_dir();
    let template = get_template(&dir, &extension.extension(), DEFAULT_RFD_TEMPLATE_PATH);
    let reserve_number = reserve_number(&dir, number, SETTINGS.get_rfd_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let rfd_path = build_path(
        &dir,
        &title,
        &formatted_reserved_number,
        extension,
        SETTINGS.get_rfd_structure(),
    );
    ensure_path(&rfd_path)?;

    // TODO: supersceded
    // TODO: reverse links

    let starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());

    let rendered = Templates::one_off(starting_content.as_str(), &context, false)?;

    let edited = edit::edit(&rendered)?;
    fs::write(&rfd_path, edited)?;

    return Ok(rfd_path);
}

pub(crate) fn reserve_rfd(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> CliResult<()> {
    let dir = SETTINGS.get_rfd_dir();
    let reserve_number = reserve_number(&dir, number, SETTINGS.get_rfd_structure())?;

    // TODO: support more than current directory
    let repo = Repository::open(".")?;
    if git::branch_exists(&repo, reserve_number).is_err() {
        return Err(git2::Error::from_str("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str())?;

    // TODO: revisit clones. Using it for now to resolve value borrowed here after move
    let created_result = new_rfd(number, title.clone(), extension);

    let message = format!(
        "{}: Adding placeholder for RFD {}",
        reserve_number,
        title.clone()
    );
    git::add_and_commit(&repo, created_result.unwrap().as_path(), message.as_str())?;
    git::push(&repo)?;

    return Ok(());
}

pub(crate) fn generate_csv() {}

pub(crate) fn graph_rfds() {
    let graph = GraphBuilder::new_named_directed("example")
        .add_node(Node::new("N0"))
        .add_node(Node::new("N1"))
        .add_edge(Edge::new("N0", "N1"))
        .build()
        .unwrap();

    let dot = Dot { graph };
}
