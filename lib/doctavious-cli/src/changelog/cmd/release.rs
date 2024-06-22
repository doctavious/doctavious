use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use glob::Pattern;
use indexmap::IndexMap;
use regex::Regex;
use scm::drivers::{Scm, ScmRepository};
use scm::ScmCommitRange;
use tracing::{trace, warn};

use crate::changelog::commit::Commit;
use crate::changelog::release::Release;
use crate::changelog::ChangelogErrors::ChangelogError;
use crate::changelog::{Changelog, ChangelogCommitSort, ChangelogRange};
use crate::settings::{load_settings, ChangelogSettings, Settings};
use crate::{CliResult, DoctaviousCliError};

// TODO: replace cwd with repositories
// TODO: range should likely be an enum
// TODO: ignore commits will live in .doctavious/changelog/.commitsignore

// TODO: do we want to allow users to specify the configuration file?
// pass in config path instead of cwd
pub struct ChangelogReleaseOptions<'a> {
    pub cwd: &'a Path,
    pub repositories: Option<Vec<PathBuf>>,
    pub prepend: Option<PathBuf>,
    pub range: Option<ChangelogRange>,
    pub include_paths: Option<Vec<Pattern>>,
    pub exclude_paths: Option<Vec<Pattern>>,
    pub topo_order: bool,
    pub sort: ChangelogCommitSort,
    pub tag_pattern: Option<Regex>,
    pub tag: Option<String>,
}

pub fn release(mut options: ChangelogReleaseOptions) -> CliResult<()> {
    let settings: Settings = load_settings(options.cwd)?.into_owned();
    let changelog_settings = settings.changelog.unwrap();

    if let Some(prepend) = options.prepend {
        options.prepend = Some(options.cwd.join(prepend));
    }

    // TODO: this should maybe go in bin/cli? Do we need cwd for anything else?
    match options.repositories.as_mut() {
        Some(repositories) => repositories
            .iter_mut()
            .for_each(|r| *r = options.cwd.join(r.clone())),
        None => options.repositories = Some(vec![options.cwd.to_path_buf()]),
    };

    let mut releases = Vec::<Release>::new();
    for repository in options.repositories.as_ref().unwrap() {
        let scm = Scm::get(&repository)?;

        // load ignore_files (new line commits to skip)
        let mut skip_commits = Vec::new();
        let ignore_commits_path = repository.join(".doctavious/changelog/.commitsignore");
        if ignore_commits_path.exists() {
            let commits = fs::read_to_string(ignore_commits_path)?
                .lines()
                .filter(|v| !(v.starts_with('#') || v.trim().is_empty()))
                .map(|v| String::from(v.trim()))
                .collect::<Vec<String>>();
            skip_commits.extend(commits);
        }

        // TODO: skip commits from CLI args

        // TODO: handle skip list - process ignore commit file. Git cliff converts them into CommitParsers that skip

        releases.extend(process_repository(&scm, &changelog_settings, &options)?);
    }

    // TODO: where to handle multiple changelog files

    // Process commits and releases for the changelog.
    let mut changelog = Changelog::new(releases, changelog_settings);

    // Print the result.
    // if args.bump || args.bumped_version {
    //
    // }

    // if let Some(ref path) = args.prepend {
    //     changelog.prepend(fs::read_to_string(path)?, &mut File::create(path)?)?;
    // }
    // if let Some(path) = args.output {
    //     let mut output: Box<dyn Write> = if path == PathBuf::from("-") {
    //         Box::new(io::stdout())
    //     } else {
    //         Box::new(File::create(path)?)
    //     };
    //     if args.context {
    //         changelog.write_context(&mut output)
    //     } else {
    //         changelog.generate(&mut output)
    //     }
    // } else if args.prepend.is_none() {
    //     changelog.generate(&mut io::stdout())
    // } else {
    //     Ok(())
    // }

    Ok(())
}

/// Processes the tags and commits for creating release entries for the changelog.
fn process_repository(
    scm: &Scm,
    changelog_settings: &ChangelogSettings,
    options: &ChangelogReleaseOptions,
) -> CliResult<Vec<Release>> {
    let topo_order = options.topo_order || changelog_settings.scm.topo_order.unwrap_or_default();
    let skip_regex = changelog_settings.scm.skip_tags.as_ref();
    let ignore_regex = changelog_settings.scm.ignore_tags.as_ref();

    let mut tags = scm
        .tags(&options.tag_pattern, topo_order)?
        .into_iter()
        .filter(|(_, name)| {
            let skip = skip_regex.map(|r| r.is_match(name)).unwrap_or_default();
            let ignore = ignore_regex
                .map(|r| {
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
        .collect::<IndexMap<String, String>>();

    // TODO: handle getting data from remote repository

    // TODO: make sure range is appropriate
    // most recent annotated tag `git describe --abbrev=0`
    // most recent tag `git describe --tags --abbrev=0`
    // git describe --tags
    // latest tag across branches `git describe --tags $(git rev-list --tags --max-count=1)`
    let mut commit_range = None;
    if let Some(range) = &options.range {
        match range {
            ChangelogRange::Current | ChangelogRange::Latest => {
                if tags.len() < 2 {
                    if let (Some(tag1), Some(tag2)) = (
                        scm.last_commit().map(|c| Some(c.id.to_string()))?,
                        tags.get_index(0).map(|(k, _)| k),
                    ) {
                        commit_range = Some(ScmCommitRange(tag1, Some(tag2.to_string())));
                    }
                } else {
                    let mut tag_index = tags.len() - 2;
                    if matches!(range, ChangelogRange::Current) {
                        if let Some(current_tag_index) =
                            scm.current_tag().as_ref().and_then(|tag| {
                                tags.iter()
                                    .enumerate()
                                    .find(|(_, (_, v))| v == &tag)
                                    .map(|(i, _)| i)
                            })
                        {
                            match current_tag_index.checked_sub(1) {
                                Some(i) => tag_index = i,
                                None => {
                                    return Err(DoctaviousCliError::ChangelogError(ChangelogError(String::from(
                                        "No suitable tags found. Maybe run with '--topo-order'?",
                                    ))));
                                }
                            }
                        } else {
                            return Err(DoctaviousCliError::ChangelogError(ChangelogError(
                                String::from("No tag exists for the current commit"),
                            )));
                        }
                    }
                    if let (Some(tag1), Some(tag2)) = (
                        tags.get_index(tag_index).map(|(k, _)| k),
                        tags.get_index(tag_index + 1).map(|(k, _)| k),
                    ) {
                        commit_range = Some(ScmCommitRange(tag1.to_string(), Some(tag2.to_string())));
                    }
                }
            }
            ChangelogRange::Unreleased => {
                if let Some(last_tag) = tags.last().map(|(k, _)| k) {
                    commit_range = Some(ScmCommitRange(last_tag.to_string(), None));
                }
            }
            ChangelogRange::Range(r) => {
                if let Some(parts) = r.to_string().split_once("..") {
                    commit_range = Some(ScmCommitRange(
                        parts.0.to_string(),
                        Some(parts.1.to_string()),
                    ))
                } else {
                    warn!("{}", format!("Unable to parse changelog range {r}"))
                }
            }
        }
    };

    let mut commits = scm.commits(
        &commit_range,
        options.include_paths.as_ref(),
        options.exclude_paths.as_ref(),
        changelog_settings.scm.limit_commits,
    )?;

    // if tag is provided update tags
    if let Some(ref tag) = options.tag {
        if let Some(commit_id) = commits.first().map(|c| c.id.to_string()) {
            match tags.get(&commit_id) {
                Some(tag) => {
                    warn!("There is already a tag ({}) for {}", tag, commit_id)
                }
                None => {
                    tags.insert(commit_id, tag.to_string());
                }
            }
        }
    }

    // TODO: this need to change if we want to support merging changelog across multiple repositories
    // probably would use HashMap then sort

    // process releases
    let mut releases = vec![Release::default()];
    let mut release_index = 0;
    for scm_commit in commits.iter().rev() {
        let commit = Commit::from(scm_commit);
        let commit_id = commit.id.to_string();
        if options.sort == ChangelogCommitSort::Newest {
            releases[release_index].commits.insert(0, commit);
        } else {
            releases[release_index].commits.push(commit);
        }

        if let Some(tag) = tags.get(&commit_id) {
            releases[release_index].version = Some(tag.to_string());
            releases[release_index].commit_id = Some(commit_id);
            releases[release_index].timestamp = if options.tag.as_deref() == Some(tag) {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs()
                    .try_into()?
            } else {
                scm_commit.timestamp
            };

            releases.push(Release::default());
            release_index += 1;
        }
    }

    // Add custom commit messages to the latest release.
    // if let Some(custom_commits) = &options.with_commit {
    //     if let Some(latest_release) = releases.iter_mut().last() {
    //         custom_commits.iter().for_each(|message| {
    //             latest_release
    //                 .commits
    //                 .push(Commit::from(message.to_string()))
    //         });
    //     }
    // }

    Ok(releases)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use scm::drivers::Scm;

    use crate::changelog::cmd::release::{release, ChangelogReleaseOptions};

    #[test]
    fn test_release() {
        release(ChangelogReleaseOptions {
            cwd: Path::new("../.."),
            repositories: None,
            prepend: None,
            range: None,
            include_paths: None,
            exclude_paths: None,
            topo_order: false,
            sort: Default::default(),
            tag_pattern: None,
            tag: None,
        })
        .unwrap();
    }
}
