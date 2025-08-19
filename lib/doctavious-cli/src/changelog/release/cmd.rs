use std::fs::File;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

use changelog::changelog::Changelog;
use changelog::commits::ScmTaggedCommits;
use changelog::entries::ChangelogEntry;
use changelog::errors::ChangelogErrors::ChangelogError;
use changelog::settings::{ChangelogCommitSort, ChangelogSettings, CommitParser};
use doctavious_std::regex::convert_to_regex;
use indexmap::IndexMap;
use regex::Regex;
use scm::commit::{ScmCommit, ScmCommitRange, ScmTag};
use scm::drivers::{Scm, ScmRepository};
use somever::Somever;
use tracing::warn;

use crate::changelog::release::configuration::{
    ChangelogRange, ChangelogReleaseOptions, StrippableChangelogSection,
};
use crate::errors::{CliResult, DoctaviousCliError};
use crate::settings::{Settings, load_settings};

pub fn release(mut options: ChangelogReleaseOptions) -> CliResult<()> {
    let settings_path = options.config_path.unwrap_or(options.cwd);
    let settings: Settings = load_settings(settings_path)?;
    let changelog_settings = settings.changelog.unwrap_or_default();

    release_with_settings(options, changelog_settings)
}

// TODO: where to handle multiple changelog files
fn release_with_settings(
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

    // if section is stripped set to None so we don't render it
    match options.strip {
        Some(StrippableChangelogSection::Header) => {
            changelog_settings.template.header = None;
        }
        Some(StrippableChangelogSection::Footer) => {
            changelog_settings.template.footer = None;
        }
        Some(StrippableChangelogSection::All) => {
            changelog_settings.template.header = None;
            changelog_settings.template.footer = None;
        }
        None => {}
    }

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

        if let Some(ref c) = options.ignore_commits {
            ignore_commits.extend(c.to_vec());
        }

        if let Some(ignore) = changelog_settings.commit.ignore.as_mut() {
            for c in ignore_commits {
                ignore.push(CommitParser {
                    field: "id".to_string(),
                    pattern: Regex::new(c.as_str())?,
                })
            }
        }

        // TODO: might make sense to move into SCM
        // TODO: depending on if there are issues with the current logic of determining which commits
        //       go with which tag we could explore using

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
            changelog_settings.commit.limit_commits,
        )?;

        let mut untagged_commits = vec![];
        for commit in commits.iter().rev() {
            untagged_commits.push(commit.to_owned());
            if let Some(tag) = tags.get(&commit.id) {
                let mut commits = std::mem::take(&mut untagged_commits);
                if options.commit_sort == ChangelogCommitSort::NewestFirst {
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
            if options.commit_sort == ChangelogCommitSort::NewestFirst {
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

    if let Some(path) = &options.prepend {
        let previous_changelog = fs::read_to_string(&path)?;
        let mut out = io::BufWriter::new(File::create(path)?);
        changelog.prepend(previous_changelog, &mut out)?;
    }

    if let Some(path) = &options.output {
        println!("{:?}", path);
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
        .unwrap_or(changelog_settings.release.tag_sort.unwrap_or_default());

    let include = options
        .tag_patterns
        .as_ref()
        .or(changelog_settings.release.tag_patterns.as_ref());
    let skip = options
        .skip_tag_patterns
        .as_ref()
        .or(changelog_settings.release.skip_tags.as_ref());

    let include_patterns = convert_to_regex(include)?;
    let skip_tag_patterns = convert_to_regex(skip)?;

    Ok(scm
        .tags(
            include_patterns.as_ref(),
            skip_tag_patterns.as_ref(),
            tag_sort,
            changelog_settings.version_suffixes.as_ref(),
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
                        scm.last_commit()?.and_then(|c| Some(c.id)),
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
    use std::{default, fs};

    use changelog::changelog::ChangelogOutputType;
    use changelog::settings::{
        ChangelogCommitSettings, ChangelogCommitSort, ChangelogReleaseConfiguration,
        ChangelogSettings, CommitStyleSettings, TemplateSettings,
    };
    use scm::drivers::git::{GitScmRepository, TagSort};
    use somever::VersioningScheme;
    use tempfile::TempDir;

    use super::{ChangelogReleaseOptions, release_with_settings};
    use crate::changelog::release::configuration::StrippableChangelogSection;

    // TODO: include header and footer templates
    // TODO: preprocessing / postprocessing?
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
                commit_sort: ChangelogCommitSort::OldestFirst,
                ignore_commits: None,
                tag_patterns: None,
                skip_tag_patterns: None,
                ignore_tag_patterns: None,
                tag: None,
                strip: None,
            },
            ChangelogSettings {
                output: Default::default(),
                template: TemplateSettings {
                    header: Some("Changelog Header".to_string()),
                    body: r###"
{% if version -%}
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else -%}
    ## [Unreleased]
{% endif -%}
{%- for release in releases -%}
    {%- for commit in release.commits -%}
- {{ commit.message }} - {{ commit.timestamp|date(format="%Y-%m-%d") }}
    {% endfor %}
{% endfor %}
"###
                    .to_string(),
                    footer: Some("Generated by Doctavious".to_string()),
                    ..Default::default()
                },
                commit: ChangelogCommitSettings {
                    commit_style: CommitStyleSettings::default(),
                    commit_preprocessors: None,
                    ignore: None,
                    group_parsers: None,
                    link_parsers: None,
                    sort_commits: None,
                    limit_commits: None,
                },
                release: ChangelogReleaseConfiguration {
                    tag_patterns: None,
                    skip_tags: None,
                    ignore_tags: None,
                    tag_sort: None,
                },
                remote: None,
                bump: None,
                protect_breaking_commits: false,
                exclude_ungrouped: false,
                commit_version: None,
                version_scheme: VersioningScheme::Semver,
                version_suffixes: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn should_be_able_to_strip_header() {
        let dir = TempDir::new().unwrap();
        let scm = GitScmRepository::init(&dir).expect("init git");
        scm.add_all().expect("Should add all files to SCM");

        let settings = ChangelogSettings {
            template: TemplateSettings {
                header: Some("Changelog Header".to_string()),
                body: "Changelog body".to_string(),
                footer: Some("Generated by Doctavious".to_string()),
                trim: false,
                post_processors: None,
            },
            ..Default::default()
        };

        let options = ChangelogReleaseOptions {
            cwd: dir.path(),
            output: Some(dir.path().join("changelog.md")),
            strip: Some(StrippableChangelogSection::Header),
            config_path: None,
            repositories: None,
            output_type: Default::default(),
            prepend: None,
            range: None,
            include_paths: None,
            exclude_paths: None,
            commit_sort: Default::default(),
            ignore_commits: None,
            tag_patterns: None,
            skip_tag_patterns: None,
            ignore_tag_patterns: None,
            tag: None,
            tag_sort: None,
        };

        release_with_settings(options, settings).unwrap();

        let changelog = fs::read_to_string(dir.path().join("changelog.md")).unwrap();
        assert!(!changelog.contains("Changelog Header"));
        assert!(changelog.contains("Generated by Doctavious"));
    }

    #[test]
    fn should_be_able_to_strip_footer() {
        let dir = TempDir::new().unwrap();
        let scm = GitScmRepository::init(&dir).expect("init git");
        scm.add_all().expect("Should add all files to SCM");

        let settings = ChangelogSettings {
            template: TemplateSettings {
                header: Some("Changelog Header".to_string()),
                body: "Changelog body".to_string(),
                footer: Some("Generated by Doctavious".to_string()),
                trim: false,
                post_processors: None,
            },
            ..Default::default()
        };

        let options = ChangelogReleaseOptions {
            cwd: dir.path(),
            config_path: None,
            repositories: None,
            output: Some(dir.path().join("changelog.md")),
            output_type: Default::default(),
            prepend: None,
            range: None,
            include_paths: None,
            exclude_paths: None,
            commit_sort: Default::default(),
            ignore_commits: None,
            tag_patterns: None,
            skip_tag_patterns: None,
            ignore_tag_patterns: None,
            tag: None,
            tag_sort: None,
            strip: Some(StrippableChangelogSection::Footer),
        };

        release_with_settings(options, settings).unwrap();
        let changelog = fs::read_to_string(dir.path().join("changelog.md")).unwrap();
        assert!(!changelog.contains("Generated by Doctavious"));
        assert!(changelog.contains("Changelog Header"));
    }

    #[test]
    fn should_be_able_to_strip_all() {
        let dir = TempDir::new().unwrap();

        let scm = GitScmRepository::init(&dir).expect("init git");
        scm.add_all().expect("Should add all files to SCM");

        release_with_settings(
            ChangelogReleaseOptions {
                cwd: dir.path(),
                config_path: None,
                repositories: None,
                output: Some(dir.path().join("changelog.md")),
                output_type: Default::default(),
                prepend: None,
                range: None,
                include_paths: None,
                exclude_paths: None,
                commit_sort: Default::default(),
                ignore_commits: None,
                tag_patterns: None,
                skip_tag_patterns: None,
                ignore_tag_patterns: None,
                tag: None,
                tag_sort: None,
                strip: Some(StrippableChangelogSection::All),
            },
            ChangelogSettings {
                template: TemplateSettings {
                    header: Some("Changelog Header".to_string()),
                    body: "Changelog body".to_string(),
                    footer: Some("Generated by Doctavious".to_string()),
                    trim: false,
                    post_processors: None,
                },
                ..Default::default()
            },
        )
        .unwrap();

        let changelog = fs::read_to_string(dir.path().join("changelog.md")).unwrap();
        assert!(!changelog.contains("Changelog Header"));
        assert!(!changelog.contains("Generated by Doctavious"));
    }

    // TODO: test multiple repositories - verify name is included

    // TODO: test output individual files
    // TODO: with specific format (asciidoc)
}
