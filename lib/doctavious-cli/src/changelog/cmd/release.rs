use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use changelog::changelog::Changelog;
use changelog::errors::ChangelogErrors::ChangelogError;
use changelog::release::Release;
use changelog::settings::ChangelogSettings;
use glob::Pattern;
use indexmap::IndexMap;
use regex::Regex;
use scm::commit::{ScmCommitRange, ScmTag, TaggedCommits};
use scm::drivers::git::TagSort;
use scm::drivers::{Scm, ScmRepository};
use tracing::{trace, warn};

use crate::changelog::settings::{ChangelogCommitSort, ChangelogRange};
use crate::errors::{CliResult, DoctaviousCliError};
use crate::settings::{load_settings, Settings};

// TODO: replace cwd with repositories
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
    pub sort: ChangelogCommitSort,

    pub skip_commits: Option<Vec<String>>,

    /// Sets the tag for the latest version
    pub tag_pattern: Option<Regex>,
    pub tag: Option<String>,
    pub tag_sort: Option<TagSort>, // TODO: this needs to fit into Somever sorting
}

pub fn release_with_settings(
    mut options: ChangelogReleaseOptions,
    changelog_settings: ChangelogSettings,
) -> CliResult<()> {
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

    // TODO: how do we want to structure tag/commits from different repositories?
    let mut tagged_commits = Vec::<TaggedCommits>::new();
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

        if let Some(ref skip) = options.skip_commits {
            skip_commits.extend(skip.to_vec());
        }

        // TODO: handle skip list - process ignore commit file. Git cliff converts them into CommitParsers that skip

        // releases.extend(process_repository(&scm, &changelog_settings, &options)?);

        let mut tags = get_tags(&scm, &changelog_settings, &options)?;
        let commit_range = determine_commit_range(&scm, &tags, &options)?;
        let mut commits = scm.commits(
            &commit_range,
            options.include_paths.as_ref(),
            options.exclude_paths.as_ref(),
            changelog_settings.scm.limit_commits,
        )?;

        // let mut tagged_commits = IndexMap::new();
        let mut untagged_commits = vec![];
        // TODO: double check if I need this reverse?
        for commit in commits.iter().rev() {
            // Sort Oldest - lists newest first to oldest
            if options.sort == ChangelogCommitSort::Newest {
                untagged_commits.insert(0, commit.to_owned());
            } else {
                untagged_commits.push(commit.to_owned());
            }

            // is this the best way to get appropriate tag for commit or something like the following
            // more appropriate
            // git describe --contains <commit>
            // ➜  cockroach git:(master) git tag --contains 1ee036d / 1ee036d42c97cc96652cb4a1588a6f481a9a62ab
            // custombuild-v24.1.0-alpha.5-1783-g6cde73f5565
            // v24.2.0-alpha.1
            // v24.2.0-alpha.2
            // v24.2.0-beta.1
            // v24.2.0-beta.2
            if let Some(tag) = tags.get(&commit.id) {
                // tagged_commits.insert(tag, untagged_commits.clone());
                tagged_commits.push(TaggedCommits {
                    tag: Some(tag.to_owned()),
                    commits: untagged_commits.clone(),
                    timestamp: commit.timestamp,
                });
                untagged_commits.clear();
            }
        }

        // handle untagged commits
        if !untagged_commits.is_empty() {
            // if tag is provided use as latest tag
            if let Some(ref tag) = options.tag {
                let latest_commit = if options.sort == ChangelogCommitSort::Newest {
                    untagged_commits.first()
                } else {
                    untagged_commits.last()
                };

                if let Some(commit) = latest_commit {
                    tagged_commits.push(TaggedCommits {
                        tag: Some(scm.get_tag(tag)),
                        commits: untagged_commits.clone(),
                        timestamp: commit.timestamp,
                    });
                }
            } else {
                tagged_commits.push(TaggedCommits {
                    tag: None,
                    commits: untagged_commits.clone(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)?
                        .as_secs()
                        .try_into()?,
                });
            }
        }
    }

    // TODO: where to handle multiple changelog files

    // Process commits and releases for the changelog.
    let mut changelog = Changelog::new(tagged_commits, changelog_settings)?;

    let mut output: Box<dyn Write> = Box::new(File::create("./test_changelog.md")?);
    changelog.generate(&mut output)?;

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

pub fn release(mut options: ChangelogReleaseOptions) -> CliResult<()> {
    let settings: Settings = load_settings(options.cwd)?.into_owned();
    let changelog_settings = settings.changelog.unwrap();

    release_with_settings(options, changelog_settings)
}

/// Processes the tags and commits for creating release entries for the changelog.
fn process_repository<'a>(
    scm: &'a Scm,
    changelog_settings: &'a ChangelogSettings,
    options: &'a ChangelogReleaseOptions,
) -> CliResult<Vec<Release>> {
    let mut tags = get_tags(scm, changelog_settings, options)?;
    // TODO: handle getting data from remote repository

    let commit_range = determine_commit_range(scm, &tags, options)?;
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
                    warn!("There is already a tag ({}) for {}", &tag.name, commit_id)
                }
                None => {
                    tags.insert(commit_id, scm.get_tag(tag));
                }
            }
        }
    }

    // TODO: support merging changelogs across multiple repositories such that if the release version
    // matches they are grouped together. Do we want more options for users to call out or group?
    // Like we could add the repo name and allow users to group by in the template if they choose?

    // process releases
    // let mut releases = vec![Release::default()];
    // let mut release_index = 0;
    // for scm_commit in commits.iter().rev() {
    //     let commit = Commit::from(scm_commit);
    //     let commit_id = commit.id.to_string();
    //
    //     // Sort Oldest - lists newest first to oldest
    //     if options.sort == ChangelogCommitSort::Newest {
    //         releases[release_index].commits.insert(0, commit);
    //     } else {
    //         releases[release_index].commits.push(commit);
    //     }
    //
    //     if let Some(tag) = tags.get(&commit_id) {
    //         releases[release_index].version = Some(tag.to_string());
    //         releases[release_index].commit_id = Some(commit_id);
    //         releases[release_index].timestamp = if options.tag.as_deref() == Some(tag) {
    //             SystemTime::now()
    //                 .duration_since(UNIX_EPOCH)?
    //                 .as_secs()
    //                 .try_into()?
    //         } else {
    //             scm_commit.timestamp
    //         };
    //
    //         releases.push(Release::default());
    //         release_index += 1;
    //     }
    // }

    // TODO: add fake commit here or if "Release item" idea pans out could add it there
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

    // Ok(releases)
    todo!()
}

fn get_tags(
    scm: &Scm,
    changelog_settings: &ChangelogSettings,
    options: &ChangelogReleaseOptions,
) -> CliResult<IndexMap<String, ScmTag>> {
    let tag_sort = options
        .tag_sort
        .unwrap_or(changelog_settings.scm.tag_sort.unwrap_or_default());
    let skip_regex = changelog_settings.scm.skip_tags.as_ref();
    let ignore_regex = changelog_settings.scm.ignore_tags.as_ref();

    Ok(scm
        .tags(&options.tag_pattern, tag_sort)?
        .into_iter()
        .filter(|(_, tag)| {
            let skip = skip_regex
                .map(|r| r.is_match(&tag.name))
                .unwrap_or_default();
            let ignore = ignore_regex
                .map(|r| {
                    if r.as_str().trim().is_empty() {
                        return false;
                    }

                    let ignore_tag = r.is_match(&tag.name);
                    if ignore_tag {
                        trace!("Ignoring release: {}", &tag.name)
                    }
                    ignore_tag
                })
                .unwrap_or_default();

            skip || !ignore
        })
        .collect::<IndexMap<String, ScmTag>>())
}

fn determine_commit_range(
    scm: &Scm,
    tags: &IndexMap<String, ScmTag>,
    options: &ChangelogReleaseOptions,
) -> CliResult<Option<ScmCommitRange>> {
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
                                    .find(|(_, (_, v))| v.name == tag.name)
                                    .map(|(i, _)| i)
                            })
                        {
                            match current_tag_index.checked_sub(1) {
                                Some(i) => tag_index = i,
                                None => {
                                    return Err(DoctaviousCliError::ChangelogError(
                                        ChangelogError(String::from("No suitable tags found")),
                                    ));
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
                        commit_range =
                            Some(ScmCommitRange(tag1.to_string(), Some(tag2.to_string())));
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
                    // TODO: this should probably be an error
                    warn!("{}", format!("Unable to parse changelog range {r}"))
                }
            }
        }
    };

    Ok(commit_range)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use changelog::settings::{ChangelogScmSettings, ChangelogSettings};
    use scm::drivers::git::TagSort;
    use somever::VersioningScheme;

    use crate::changelog::cmd::release::{release, release_with_settings, ChangelogReleaseOptions};

    #[test]
    fn test_release() {
        release_with_settings(
            ChangelogReleaseOptions {
                cwd: Path::new("../.."),
                repositories: None,
                prepend: None,
                range: None,
                include_paths: None,
                exclude_paths: None,
                tag_sort: Some(TagSort::default()),
                sort: Default::default(),
                skip_commits: None,
                tag_pattern: None,
                tag: None,
            },
            ChangelogSettings {
                scm: ChangelogScmSettings {
                    commit_version: None,
                    commit_style: None,
                    commit_preprocessors: None,
                    skips: None,
                    group_parsers: None,
                    link_parsers: None,
                    protect_breaking_commits: None,
                    filter_commits: Some(false),
                    skip_tags: None,
                    ignore_tags: None,
                    tag_sort: None,
                    sort_commits: None,
                    limit_commits: None,
                    version_scheme: VersioningScheme::Semver,
                },
                remote: None,
                bump: None,
            },
        )
        .unwrap();
    }
}
