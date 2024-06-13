use std::fs;
use std::path::Path;

use glob::Pattern;
use tracing::trace;
use scm::drivers::Scm;
use scm::ScmRepository;

use crate::settings::{load_settings, Settings};
use crate::CliResult;

// TODO: replace cwd with repositories
// TODO: range should likely be an enum
// TODO: ignore commits will live in .doctavious/changelog/.commitsignore
pub fn release(
    cwd: &Path,
    range: Option<String>,
    include_paths: Option<Vec<Pattern>>,
    exclude_paths: Option<Vec<Pattern>>,
    topo_order: bool,
) -> CliResult<()> {
    // get all configuration
    let settings: Settings = load_settings(cwd)?.into_owned();
    let changelog_settings = settings.changelog.unwrap();
    let topo_order = topo_order || changelog_settings.scm.topo_order.unwrap_or_default();
    let skip_regex = changelog_settings.scm.skip_tags;
    let ignore_regex = changelog_settings.scm.ignore_tags;

    // TODO: support multiple repositories
    let scm = Scm::get(cwd)?;
    // load ignore_files (new line commits to skip)
    let ignore_commits_path = cwd.join(".doctavious/changelog/.commitsignore");
    let ignored_commits = if ignore_commits_path.exists() {
        fs::read_to_string(ignore_commits_path)
            .and_then(|s| s.lines().collect())
            .unwrap_or_default()
    } else {
        vec![]
    };

    // TODO: process ignore commit file. Git cliff converts them into CommitParsers that skip

    let tags = scm
        .tags(&range, topo_order)?
        .into_iter()
        .filter(|(_, name)| {
            let skip = skip_regex.map(|r| r.is_match(name)).unwrap_or_default();

            let ignore = ignore_regex.map(|r| {
                if r.as_str().trim().is_empty() {
                    return false;
                }

                let ignore_tag = r.is_match(name);
                if ignore_tag {
                    trace!("Ignoring release: {}", name)
                }
                ignore_tag
            })
            .unwrap_or_default();

            skip || !ignore
        })
        .collect();

    // get range
    // let unreleased = false;
    // let latest = false;
    // let current = false;

    // // TODO: see how latest vs current works
    // // get range
    // if unreleased {
    // } else if latest || current {
    // }
    //

    // optionally limit commits
    let mut commits = scm.commits(&range, include_paths.as_ref(), exclude_paths.as_ref())?;
    if include_paths.is_some() || exclude_paths.is_some() {
        commits.retain(|commit| {
            // include_path / exclude_path

            // if let Some(include_path) = &include_paths {
            //     // include_path
            //     //     .iter()
            //     //     .any(|glob| glob.matches_path(new_file_path))
            // } else if let Some(exclude_path) = &exclude_paths {
            //     // !exclude_path
            //     //     .iter()
            //     //     .any(|glob| glob.matches_path(new_file_path))
            // } else {
            //     false
            // }

            true
        });
    }
    // if tag is provided update tags

    // process releases
    for c in commits.iter().rev() {
        //
        //     // assign commits to releases
        //
        //     // if sort
        //     // releases[release_index].commits.insert(0, commit);
        //     // else
        //     // releases[release_index].commits.push(commit);
        //
        //     // if let Some(tag) = tags.get(&commit_id) {
        //
        //     // custom commit messages
    }

    // let mut releases = Vec::<Release>::new();

    // Process commits and releases for the changelog.
    // let mut changelog = Changelog::new(releases)?;

    // where to handle multiple changelog files

    // Print the result.
    // if args.bump || args.bumped_version {
    //
    // }

    // if prepend

    // if output
    // else to std out

    Ok(())
}
