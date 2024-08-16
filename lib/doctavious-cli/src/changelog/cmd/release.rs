use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use changelog::changelog::{Changelog, ChangelogOutputType};
use changelog::entries::{ChangelogCommit, ChangelogEntry};
use changelog::errors::ChangelogErrors::ChangelogError;
use changelog::release::Release;
use changelog::settings::ChangelogSettings;
use glob::Pattern;
use indexmap::IndexMap;
use regex::Regex;
use scm::commit::{ScmCommitRange, ScmTag, TaggedCommits};
use scm::drivers::git::TagSort;
use scm::drivers::{Scm, ScmRepository};
use somever::Somever;
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
    pub output: Option<PathBuf>,
    pub output_type: ChangelogOutputType,
    pub prepend: Option<PathBuf>,
    pub range: Option<ChangelogRange>,
    pub include_paths: Option<Vec<Pattern>>,
    pub exclude_paths: Option<Vec<Pattern>>,
    pub sort: ChangelogCommitSort,

    pub skip_commits: Option<Vec<String>>,

    /// Sets the tag for the latest version
    pub tag_pattern: Option<Regex>,
    pub tag: Option<String>,

    // TODO: this needs to fit into Somever sorting
    pub tag_sort: Option<TagSort>,
}

// TODO: where to handle multiple changelog files
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

            if let Some(tag) = tags.get(&commit.id) {
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

    // Process commits and releases for the changelog.
    let mut changelog = Changelog::new(tagged_commits, changelog_settings)?;

    // TODO: does it make sense to allow prepend and output together?
    if let Some(path) = &options.prepend {
        let mut previous_changelog = fs::read_to_string(&path)?;
        // remove header in previous changelog
        // if let Some(header) = &self.config.changelog.header {
        //     previous_changelog = previous_changelog.replacen(header, "", 1);
        // }

        let mut output = File::create(&path)?;
        changelog.generate(&mut output)?;

        // write previous changelog back
        write!(output, "{previous_changelog}")?;
    }

    if let Some(path) = &options.output {
        let mut output = File::create(path)?;
        changelog.generate(&mut output)?;
    } else if options.prepend.is_none() {
        changelog.generate(&mut io::stdout())?;
    }

    Ok(())
}

pub fn release(mut options: ChangelogReleaseOptions) -> CliResult<()> {
    let settings: Settings = load_settings(options.cwd)?.into_owned();
    let changelog_settings = settings.changelog.unwrap();

    release_with_settings(options, changelog_settings)
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
        .tags(
            &options.tag_pattern,
            tag_sort,
            changelog_settings.scm.version_suffixes.as_ref(),
        )?
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

pub struct VersionUpdater {
    pub features_always_increment_minor: bool,
    pub breaking_always_increment_major: bool,
    // TODO: could this be the same as our group / commit parser?
    pub major_increment_regex: Option<Regex>,
    pub minor_increment_regex: Option<Regex>,
}

impl VersionUpdater {
    pub fn increment<I>(&self, version: &Somever, commit_entries: &[ChangelogEntry]) {
        let breaking_change = commit_entries.iter().any(|e| e.commit.breaking);

        let is_major_bump = || {
            (breaking_change
                || Self::is_there_a_custom_match(
                    self.major_increment_regex.as_ref(),
                    commit_entries,
                ))
                && (version.major() != 0 || self.breaking_always_increment_major)
        };

        // let is_minor_bump = || {
        //     let is_feat_bump = || {
        //         is_there_a_feature()
        //             && (version.major() != 0 || self.features_always_increment_minor)
        //     };
        //     let is_breaking_bump =
        //         || version.major() == 0 && version.minor() != 0 && breaking_change;
        //     is_feat_bump()
        //         || is_breaking_bump()
        //         || Self::is_there_a_custom_match(self.custom_minor_increment_regex.as_ref(), commit_entries)
        // };
    }

    fn is_there_a_custom_match(regex_option: Option<&Regex>, commits: &[ChangelogEntry]) -> bool {
        if let Some(regex) = regex_option {
            commits
                .iter()
                .any(|commit| Self::custom_commit_matches_regex(regex, commit))
        } else {
            false
        }
    }

    fn custom_commit_matches_regex(regex: &Regex, commit: &ChangelogEntry) -> bool {
        // if let CommitType::Custom(custom_type) = &commit.commit_type {
        //     regex.is_match(custom_type)
        // } else {
        //     false
        // }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use std::path::{Path, PathBuf};

    use changelog::changelog::ChangelogKind;
    use changelog::settings::{ChangelogScmSettings, ChangelogSettings, TemplateSettings};
    use scm::drivers::git::TagSort;
    use somever::VersioningScheme;

    use crate::changelog::cmd::release::{release, release_with_settings, ChangelogReleaseOptions};

    #[test]
    fn test_release() {
        release_with_settings(
            ChangelogReleaseOptions {
                cwd: Path::new("../.."),
                repositories: None,
                output: Some(PathBuf::from("./test_changelog.md")),
                output_type: ChangelogKind::Single,
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
                structure: ChangelogKind::Single,
                format: Default::default(),
                templates: TemplateSettings {
                    body: r###"
{% if version -%}
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else -%}
    ## [Unreleased]
{% endif -%}
{% for commit in commits -%}
    - {{ commit.message }}
{% endfor %}
"###
                    .to_string(),
                    ..Default::default()
                },
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
                    version_suffixes: None,
                },
                remote: None,
                bump: None,
            },
        )
        .unwrap();
    }
}
