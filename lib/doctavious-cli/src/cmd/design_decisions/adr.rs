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
    init_dir, load_settings, persist_settings, AdrSettings, DEFAULT_ADR_DIR,
    DEFAULT_ADR_TEMPLATE_PATH, INIT_ADR_TEMPLATE_PATH, SETTINGS,
};
use crate::templates::{get_template, TemplateContext, Templates};
use crate::{edit, git, CliResult};

pub(crate) fn init_adr(
    directory: Option<String>,
    structure: FileStructure,
    extension: Option<MarkupFormat>,
) -> CliResult<PathBuf> {
    let mut settings = load_settings().unwrap_or_else(|_| Default::default());

    let dir = match directory {
        None => DEFAULT_ADR_DIR,
        Some(ref d) => d,
    };

    let adr_settings = AdrSettings {
        dir: Some(dir.to_string()),
        structure: Some(structure),
        template_extension: extension,
    };

    settings.adr_settings = Some(adr_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    // TODO: This seems a bit unnecessary for init which is pretty much static content outside of date
    return new_adr(
        Some(1),
        "Record Architecture Decisions".to_string(),
        SETTINGS.get_adr_template_extension(extension),
        INIT_ADR_TEMPLATE_PATH,
    );
}

pub(crate) fn new_adr(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
    template_path: &str,
    // supercedes: Option<Vec<String>>,
    // links: Option<Vec<String>>
) -> CliResult<PathBuf> {
    let dir = SETTINGS.get_adr_dir();
    let template = get_template(&dir, &extension.extension(), template_path);
    let reserve_number = reserve_number(&dir, number, SETTINGS.get_adr_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let adr_path = build_path(
        &dir,
        &title,
        &formatted_reserved_number,
        extension,
        SETTINGS.get_adr_structure(),
    );
    ensure_path(&adr_path)?;

    // TODO: supersceded
    // if let Some(targets) = supercedes {
    //     for target in targets {
    //         // "$adr_bin_dir/_adr_add_link" "$target" "Superceded by" "$dstfile"
    //         // "$adr_bin_dir/_adr_remove_status" "Accepted" "$target"
    //         // "$adr_bin_dir/_adr_add_link" "$dstfile" "Supercedes" "$target"
    //     }
    // }

    // TODO: reverse links
    // if let Some(others) = links {
    //     for other in others {
    //         // target="$(echo $l | cut -d : -f 1)"
    //         // forward_link="$(echo $l | cut -d : -f 2)"
    //         // reverse_link="$(echo $l | cut -d : -f 3)"

    //         // "$adr_bin_dir/_adr_add_link" "$dstfile" "$forward_link" "$target"
    //         // "$adr_bin_dir/_adr_add_link" "$target" "$reverse_link" "$dstfile"
    //     }
    // }

    let starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    // TODO: allow date to be customized
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());

    let rendered = Templates::one_off(starting_content.as_str(), &context, false)?;

    let edited = edit::edit(&rendered)?;
    fs::write(&adr_path, edited)?;
    return Ok(adr_path);
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
pub(crate) fn reserve_adr(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> CliResult<()> {
    let dir = SETTINGS.get_adr_dir();
    let reserve_number = reserve_number(&dir, number, SETTINGS.get_adr_structure())?;

    // TODO: support more than current directory
    let repo = Repository::open(".")?;
    if git::branch_exists(&repo, reserve_number) {
        return Err(git2::Error::from_str("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str());

    // TODO: revisit clones. Using it for now to resolve value borrowed here after move
    let created_result = new_adr(number, title.clone(), extension, DEFAULT_ADR_TEMPLATE_PATH);

    let message = format!(
        "{}: Adding placeholder for ADR {}",
        reserve_number,
        title.clone()
    );
    git::add_and_commit(&repo, created_result.unwrap().as_path(), message.as_str());
    git::push(&repo);

    return Ok(());
}

pub(crate) fn generate_csv() {}

pub(crate) fn graph_adrs() {
    let graph = GraphBuilder::new_named_directed("example")
        .add_node(Node::new("N0"))
        .add_node(Node::new("N1"))
        .add_edge(Edge::new("N0", "N1"))
        .build()
        .unwrap();

    let dot = Dot { graph };
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::cmd::design_decisions::adr::init_adr;
    use crate::file_structure::FileStructure;
    use crate::markup_format::MarkupFormat;

    // init default
    #[test]
    fn init() {
        let dir = tempdir().unwrap();

        init_adr(
            Some(dir.path().display().to_string()),
            FileStructure::default(),
            Some(MarkupFormat::default()),
        )
        .expect("should init adr");

        dir.close().unwrap();
    }

    // init options

    // init override existing

    // new w/o init

    // new with init
}
