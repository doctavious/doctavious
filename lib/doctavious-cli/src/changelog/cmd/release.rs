use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use changelog::changelog::{Changelog, ChangelogOutputType};
use changelog::commits::ScmTaggedCommits;
use changelog::entries::{ChangelogCommit, ChangelogEntry};
use changelog::errors::ChangelogErrors::ChangelogError;
use changelog::release::Release;
use changelog::settings::{ChangelogSettings, CommitParser};
use doctavious_std::regex::convert_to_regex;
use glob::Pattern;
use indexmap::IndexMap;
use regex::{Regex, RegexBuilder};
use scm::commit::{ScmCommit, ScmCommitRange, ScmTag};
use scm::drivers::git::TagSort;
use scm::drivers::{Scm, ScmRepository};
use serde::{Deserialize, Serialize};
use somever::Somever;
use strum::{Display, EnumString, VariantNames};
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
    pub config_path: Option<&'a Path>,
    pub repositories: Option<Vec<PathBuf>>,
    pub output: Option<PathBuf>,
    pub output_type: ChangelogOutputType,
    pub prepend: Option<PathBuf>,
    pub range: Option<ChangelogRange>,
    pub include_paths: Option<Vec<Pattern>>,
    pub exclude_paths: Option<Vec<Pattern>>,
    pub commit_sort: ChangelogCommitSort,

    pub ignore_commits: Option<Vec<String>>,

    /// Changes to these will be retained
    pub tag_patterns: Option<Vec<String>>,

    /// Changes belonging to these releases will be included in the next non-skipped release
    pub skip_tag_patterns: Option<Vec<String>>,

    /// Changes belonging to these releases will not appear in the changelog
    pub ignore_tag_patterns: Option<Vec<String>>,

    pub tag: Option<String>,

    // TODO: this needs to fit into Somever sorting
    pub tag_sort: Option<TagSort>,
}

#[remain::sorted]
#[derive(
    Clone, Debug, Display, Default, Deserialize, EnumString, Eq, PartialEq, Serialize, VariantNames,
)]
pub enum BumpOption {
    #[default]
    Auto,
    Specific(BumpType),
}

/// Version bump type.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, PartialEq, Serialize, VariantNames,
)]
pub enum BumpType {
    /// Bump major version.
    Major,
    /// Bump minor version.
    Minor,
    /// Bump patch version.
    #[default]
    Patch,
}

pub fn release(mut options: ChangelogReleaseOptions) -> CliResult<()> {
    let settings_path = options.config_path.unwrap_or(options.cwd);
    let settings: Settings = load_settings(settings_path)?;
    let changelog_settings = settings.changelog.unwrap_or_default();

    release_with_settings(options, changelog_settings)
}

// TODO: where to handle multiple changelog files
pub fn release_with_settings(
    mut options: ChangelogReleaseOptions,
    mut changelog_settings: ChangelogSettings,
) -> CliResult<()> {
    if let Some(prepend) = options.prepend {
        options.prepend = Some(options.cwd.join(prepend));
    }

    match options.repositories.as_mut() {
        Some(repositories) => repositories
            .iter_mut()
            .for_each(|r| *r = options.cwd.join(r.clone())),
        None => options.repositories = Some(vec![options.cwd.to_path_buf()]),
    };

    // TODO: how do we want to structure tag/commits from different repositories?
    // Do we want to include a label for the repo?
    // TODO: configuration for unified?
    // Could this be handled by a groupby in the template?
    let mut tagged_commits = Vec::<ScmTaggedCommits>::new();
    for repository in options.repositories.as_ref().unwrap() {
        let scm = Scm::get(&repository)?;

        // load ignore_files (new line commits to skip)
        let mut ignore_commits = Vec::new();
        // TODO: const for .commitsignore path
        let ignore_commits_path = repository.join(".doctavious/changelog/.commitsignore");
        if ignore_commits_path.exists() {
            let commits = fs::read_to_string(ignore_commits_path)?
                .lines()
                .filter(|v| !(v.starts_with('#') || v.trim().is_empty()))
                .map(|v| String::from(v.trim()))
                .collect::<Vec<String>>();
            ignore_commits.extend(commits);
        }

        if let Some(ref skip) = options.ignore_commits {
            ignore_commits.extend(skip.to_vec());
        }

        if let Some(skips) = changelog_settings.scm.ignore.as_mut() {
            for skip_commit in ignore_commits {
                skips.push(CommitParser {
                    field: "id".to_string(),
                    pattern: Regex::new(skip_commit.as_str())?,
                })
            }
        }

        // TODO: might make sense to move into SCM
        // TODO: depending on if there are issues with the current log of determining which commits
        // go with which tag we could explore using
        // git tag --contains <commit>  -- this returns a tag
        // git rev-list -1  <tag>   -- this returns a commit
        // and then iterating up to that commit and repeating the process
        // could potentially use `git describe --contains <commit>` but need to be careful about the
        // output as it contains additional information outside of the main tag that we would need to
        // parse. See https://stackoverflow.com/questions/62588666/how-to-interpret-the-output-of-git-describe-contains
        // One bad thing about `git tag contains` is that it only allows inclusive pattern
        let mut tags = get_tags(&scm, &changelog_settings, &options)?;
        let commit_range = determine_commit_range(&scm, &tags, &options)?;

        let mut commits = scm.commits(
            commit_range.as_ref(),
            options.include_paths.as_ref(),
            options.exclude_paths.as_ref(),
            changelog_settings.scm.limit_commits,
        )?;

        let mut untagged_commits = vec![];
        for commit in commits.iter().rev() {
            untagged_commits.push(commit.to_owned());
            if let Some(tag) = tags.get(&commit.id) {
                let mut commits = std::mem::take(&mut untagged_commits);
                if options.commit_sort == ChangelogCommitSort::Newest_First {
                    commits.reverse();
                }

                tagged_commits.push(ScmTaggedCommits {
                    repository: repository
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    tag: Some(tag.clone()), // might be able to get ownership with tags.shift_remove
                    commits,
                    timestamp: Some(tag.timestamp),
                });
            }
        }

        // handle untagged commits
        if !untagged_commits.is_empty() {
            // if tag is provided use as latest tag
            let mut timestamp = None;
            let tag = if let Some(ref tag) = options.tag {
                let mut scm_tag = scm.get_tag(tag);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs()
                    .try_into()?;

                timestamp = Some(now);
                scm_tag.timestamp = now;

                Some(scm_tag)
            } else {
                None
            };

            let mut commits = std::mem::take(&mut untagged_commits);
            if options.commit_sort == ChangelogCommitSort::Newest_First {
                commits.reverse();
            }

            tagged_commits.push(ScmTaggedCommits {
                repository: repository
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                tag,
                commits,
                timestamp,
            });
        }
    }

    // Process commits and releases for the changelog.
    let mut changelog = Changelog::new(tagged_commits, changelog_settings)?;

    // TODO: does it make sense to allow prepend and output together?
    if let Some(path) = &options.prepend {
        let previous_changelog = fs::read_to_string(&path)?;
        let mut out = io::BufWriter::new(File::create(path)?);
        changelog.prepend(previous_changelog, &mut out)?;
    }

    if let Some(path) = &options.output {
        let mut output = File::create(path)?;
        changelog.generate(&mut output)?;
    } else if options.prepend.is_none() {
        changelog.generate(&mut io::stdout())?;
    }

    Ok(())
}

fn get_tags(
    scm: &Scm,
    changelog_settings: &ChangelogSettings,
    options: &ChangelogReleaseOptions,
) -> CliResult<IndexMap<String, ScmTag>> {
    let tag_sort = options
        .tag_sort
        .unwrap_or(changelog_settings.scm.tag_sort.unwrap_or_default());

    let include_patterns = convert_to_regex(options.tag_patterns.as_ref())?;
    let skip_tag_patterns = convert_to_regex(changelog_settings.scm.skip_tags.as_ref())?;

    Ok(scm
        .tags(
            include_patterns.as_ref(),
            skip_tag_patterns.as_ref(),
            tag_sort,
            changelog_settings.scm.version_suffixes.as_ref(),
        )?
        .into_iter()
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

pub struct TCommits {
    pub repository_name: String,
    pub tag: Option<ScmTag>,
    pub commits: Vec<ScmCommit>,
    pub timestamp: Option<i64>,
}

#[cfg(test)]
mod tests {
    use std::default::Default;
    use std::path::{Path, PathBuf};

    use changelog::changelog::ChangelogOutputType;
    use changelog::settings::{ChangelogScmSettings, ChangelogSettings, TemplateSettings};
    use scm::drivers::git::TagSort;
    use somever::VersioningScheme;

    use crate::changelog::cmd::release::{release_with_settings, ChangelogReleaseOptions};
    use crate::changelog::settings::ChangelogCommitSort;

    #[test]
    fn test_release() {
        release_with_settings(
            ChangelogReleaseOptions {
                cwd: Path::new("../.."),
                config_path: None,
                repositories: None,
                output: Some(PathBuf::from("./test_changelog.md")),
                output_type: ChangelogOutputType::Single,
                prepend: None,
                range: None,
                include_paths: None,
                exclude_paths: None,
                tag_sort: Some(TagSort::default()),
                commit_sort: ChangelogCommitSort::Oldest_First,
                ignore_commits: None,
                tag_patterns: None,
                skip_tag_patterns: None,
                ignore_tag_patterns: None,
                tag: None,
            },
            ChangelogSettings {
                output_type: ChangelogOutputType::Single,
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
                    ignore: None,
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
