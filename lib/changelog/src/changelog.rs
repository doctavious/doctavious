use std::io::Write;

use doctavious_templating::{TemplateContext, Templates};
use git_conventional::{Commit as GitConventionalCommit, Error};
use scm::commit::{ScmCommit, TaggedCommits};
use somever::{Calver, Somever, VersioningScheme};
use strum::IntoEnumIterator;
use tracing::{debug, trace, warn};

use crate::conventional::ConventionalCommit;
use crate::entries::{ChangelogCommit, ChangelogEntry, Link};
use crate::errors::{ChangelogErrors, ChangelogResult};
use crate::release::Release;
use crate::release_notes::ReleaseNote;
use crate::settings::{
    ChangelogScmSettings, ChangelogSettings, CommitStyleSettings, GroupParser, LinkParser,
};

// Not sure about the name but essentially controls if changelog should write details to a single
// file or if they should be separated.
// Might be able to solve this just with specifying range/tags/commits and passing in a specific output file
#[non_exhaustive]
#[remain::sorted]
#[derive(Default)]
pub enum ChangelogKind {
    Multiple,
    #[default]
    Single,
}

#[remain::sorted]
enum ChangeLogFormat {
    /// changelog similar to cocogitto's format.
    Cocogitto,

    /// changelog that contains links to the commits.
    Detailed,

    /// changelog in the GitHub's format.
    GitHub,

    /// combination of the previous two formats.
    GitHubKeepAChangelog,

    /// changelog in Keep a Changelog format.
    KeepAChangelog,

    Minimal,

    /// changelog with commits are grouped by their scopes.
    Scoped,

    /// changelog with commits grouped by their scopes and sorted by group.
    ScopedSorted,

    /// changelog for unconventional commits.
    Unconventional,
}

#[derive(Debug)]
pub struct Changelog {
    releases: Vec<Release>,
    // settings: &'a ChangeLogSettings
    // body_template:   Template,
    // footer_template: Option<Template>,
    // config:          &'a Config,
}

impl Changelog {
    pub fn new(
        mut tagged_commits: Vec<TaggedCommits>,
        settings: ChangelogSettings,
    ) -> ChangelogResult<Self> {
        let filter_commits = settings.scm.filter_commits.unwrap_or_default();
        Changelog::preprocess(&mut tagged_commits, &settings);

        let commit_style_settings = settings.scm.commit_style.unwrap_or_default();
        let mut releases: Vec<Release> = vec![];

        for tagged_commit in tagged_commits {
            let mut changelog_entries = Vec::new();
            for mut commit in tagged_commit.commits {
                let mut changelog_entry_commits = vec![];
                match &commit_style_settings {
                    CommitStyleSettings::Conventional(settings) => {
                        let conventional_commit = GitConventionalCommit::parse(&commit.message);
                        let c = match conventional_commit {
                            Ok(conv) => ChangelogCommit::from_conventional(ConventionalCommit {
                                commit: commit.clone(),
                                conv,
                            }),
                            Err(e) => {
                                if settings.include_unconventional.unwrap_or_default() {
                                    ChangelogCommit::from_scm_commit(&commit);
                                }

                                return Err(ChangelogErrors::ChangelogError(e.to_string()));
                            }
                        };

                        changelog_entry_commits.push(c);
                    }
                    CommitStyleSettings::ReleaseNote(_) => {
                        let release_notes = ReleaseNote::parse_commit(&commit);
                        for release_note in release_notes {
                            changelog_entry_commits
                                .push(ChangelogCommit::from_release_note(&release_note));
                        }
                    }
                    CommitStyleSettings::Standard(_) => {
                        changelog_entry_commits.push(ChangelogCommit::from_scm_commit(&commit));
                    }
                };

                let protect_breaking = settings.scm.protect_breaking_commits.unwrap_or_default();
                'commits: for commit in changelog_entry_commits {
                    if let Some(skips) = &settings.scm.skips {
                        for skip in skips {
                            if skip.is_match(&commit)? && !(commit.breaking && protect_breaking) {
                                debug!("skipping commit {}", &commit.id);
                                continue 'commits;
                            }
                        }
                    }

                    let mut entry = ChangelogEntry::new(
                        commit,
                        settings.scm.group_parsers.as_ref(),
                        settings.scm.link_parsers.as_ref(),
                    )?;

                    if entry.matched_group_parser || !filter_commits {
                        changelog_entries.push(entry);
                    } else if filter_commits {
                        trace!("Commit {} does not belong to any group", &entry.id())
                    }
                }
            }

            let tag = tagged_commit.tag;
            let version = if let Some(name) = tag.as_ref().map(|t| t.name.clone()) {
                Some(Somever::new(&settings.scm.version_scheme, &name)?)
            } else {
                None
            };

            releases.push(Release {
                version,
                tag_id: tag.as_ref().and_then(|t| t.id.clone()),
                commits: changelog_entries,
                timestamp: tagged_commit.timestamp,
            });
        }

        // TODO: sort releases based on settings
        // TODO: handle suffix

        Ok(Self { releases })
    }

    fn preprocess(mut tagged_commits: &mut Vec<TaggedCommits>, settings: &ChangelogSettings) {
        tagged_commits.iter_mut().for_each(|tagged| {
            tagged.commits = tagged
                .commits
                .iter()
                .cloned()
                .filter_map(|mut commit| {
                    if let Some(preprocessors) = &settings.scm.commit_preprocessors {
                        for preprocessor in preprocessors {
                            if let Err(e) = preprocessor.replace(&mut commit.message, vec![]) {
                                warn!(
                                    "{} - {} ({})",
                                    commit.id[..7].to_string(),
                                    e,
                                    &commit.message
                                );
                                // TODO: would we prefer to fail after first or just warn and apply all preprocessors
                                return None;
                            }
                        }
                    }
                    Some(commit)
                })
                .flat_map(|commit| {
                    let commit_style_settings =
                        settings.scm.commit_style.clone().unwrap_or_default();
                    if commit_style_settings.split_lines() {
                        commit
                            .message
                            .lines()
                            .filter_map(|line| {
                                if !line.is_empty() {
                                    let mut c = commit.clone();
                                    c.message = line.to_string();
                                    Some(c)
                                } else {
                                    None
                                }
                            })
                            .collect()
                    } else {
                        vec![commit]
                    }
                })
                .collect::<Vec<ScmCommit>>()
        })
    }

    // TODO: process_releases

    // TODO: generate
    pub fn generate<W: Write>(&self, out: &mut W) -> ChangelogResult<()> {
        // TODO: post processors

        // TODO: headers

        // TODO: support versions in different files
        for release in &self.releases {
            let template = r###"
{% if version -%}
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else -%}
    ## [Unreleased]
{% endif -%}
{% for commit in commits -%}
    - {{ commit.message }}
{% endfor %}
"###;
            let context = TemplateContext::from_serialize(release)?;
            let rendered = Templates::one_off(template, context, false)?;
            write!(out, "{}", rendered)?;
        }

        // TODO: footers

        Ok(())
    }

    // TODO: prepend

    // Increments the version for the unreleased changes
    pub fn bump_version(&self) {
        if let Some(mut release) = self.releases.first() {
            if let Some(version) = &release.version {}
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn changelog_generator() {}
}
