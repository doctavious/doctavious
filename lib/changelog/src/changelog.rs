use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use doctavious_std::regex::convert_to_regex;
use doctavious_templating::{TemplateContext, Templates};
use git_conventional::{Commit as GitConventionalCommit, Error};
use markup::MarkupFormat;
use regex::{Regex, RegexBuilder};
use scm::commit::ScmCommit;
use serde_derive::{Deserialize, Serialize};
use somever::{Calver, Somever, VersioningScheme};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, VariantNames};
use tracing::{debug, trace, warn};

use crate::commits::ScmTaggedCommits;
use crate::conventional::ConventionalCommit;
use crate::entries::{ChangelogCommit, ChangelogEntry, Link};
use crate::errors::{ChangelogErrors, ChangelogResult};
use crate::release::{Release, Releases};
use crate::release_notes::{ReleaseNote, ReleaseNotes};
use crate::settings::{
    ChangelogCommitSettings, ChangelogSettings, CommitProcessor, CommitStyleSettings, GroupParser,
    LinkParser,
};

// Not sure about the name but essentially controls if changelog should write details to a single
// file or if they should be separated.
// Might be able to solve this just with specifying range/tags/commits and passing in a specific output file
#[non_exhaustive]
#[remain::sorted]
#[derive(Clone, Debug, Display, Default, Deserialize, EnumString, Serialize, VariantNames)]
#[serde(rename_all = "lowercase")]
pub enum ChangelogOutputType {
    Individual,
    #[default]
    Single,
}

#[derive(Debug)]
pub struct Changelog {
    releases: Vec<Release>,
    // TODO: should we have a Template struct?
    // TODO: should be within settings? Perhaps a `templates` section
    header_template: Option<String>,
    body_template: String,
    footer_template: Option<String>,
    trim: bool,
    post_processors: Option<Vec<CommitProcessor>>,
    // additional_context: HashMap<String, serde_json::Value>,
}

impl Changelog {
    pub fn new(
        mut tagged_commits: Vec<ScmTaggedCommits>,
        settings: ChangelogSettings,
    ) -> ChangelogResult<Self> {
        tagged_commits = Self::ignore_tags(tagged_commits, settings.release.ignore_tags.as_ref())?;

        let mut releases: Vec<Release> = vec![];
        for mut tagged_commit in tagged_commits {
            Self::preprocess(&mut tagged_commit, &settings);
            let changelog_entries = Self::process(&mut tagged_commit, &settings)?;

            // TODO: Should we have a ReleaseVersion which could be None as unreleased
            //       or a version / tag associated?
            let tag = tagged_commit.tag;
            let version = if let Some(name) = tag.as_ref().map(|t| t.name.to_string()) {
                Some(Somever::new(settings.version_scheme, &name)?)
            } else {
                None
            };

            releases.push(Release {
                version,
                tag_id: tag.as_ref().and_then(|t| t.id.clone()),
                repository: tagged_commit.repository,
                commits: changelog_entries,
                timestamp: tagged_commit.timestamp,
            });
        }

        // TODO: add option to sort by version vs time
        //       should it replace the option for sorting tags? Is that still beneficial?
        Self::sort_releases(&mut releases, settings.version_suffixes.as_ref());

        Ok(Self {
            releases,
            header_template: settings.template.header,
            body_template: settings.template.body,
            footer_template: settings.template.footer,
            trim: settings.template.trim,
            post_processors: settings.template.post_processors,
        })
    }

    fn ignore_tags(
        tagged_commits: Vec<ScmTaggedCommits>,
        ignore: Option<&Vec<String>>,
    ) -> ChangelogResult<Vec<ScmTaggedCommits>> {
        let ignore_tags = convert_to_regex(ignore)?;
        Ok(tagged_commits
            .into_iter()
            .filter(|tc| {
                if let Some(tag) = &tc.tag {
                    let id = tag.clone().id.unwrap_or_default();
                    if let Some(ignores) = &ignore_tags {
                        for ignore in ignores {
                            if ignore.is_match(&id) {
                                return false;
                            }
                        }
                    }
                }
                true
            })
            .collect())
    }

    fn preprocess(mut tagged_commits: &mut ScmTaggedCommits, settings: &ChangelogSettings) {
        let commit_preprocessors = &settings.commit.commit_preprocessors;
        let commit_style_settings = &settings.commit.commit_style;
        tagged_commits.commits = tagged_commits
            .commits
            .iter()
            .cloned()
            .filter_map(|mut commit| {
                if let Some(preprocessors) = commit_preprocessors {
                    // TODO: would we prefer to fail after first or just warn and apply all preprocessors
                    if let Err(e) = preprocessors.iter().try_for_each(|preprocessor| {
                        preprocessor.replace(&mut commit.message, vec![])?;
                        Ok::<(), ChangelogErrors>(())
                    }) {
                        warn!(
                            "{} - {} ({})",
                            commit.id[..7].to_string(),
                            e,
                            &commit.message
                        );
                        return None;
                    }
                }
                Some(commit)
            })
            .flat_map(|commit| {
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
    }

    fn process(
        tagged_commits: &ScmTaggedCommits,
        settings: &ChangelogSettings,
    ) -> ChangelogResult<Vec<ChangelogEntry>> {
        let mut changelog_entries = Vec::new();

        let commit_style_settings = &settings.commit.commit_style;
        let protect_breaking = settings.protect_breaking_commits;
        let filter_commits = settings.exclude_ungrouped;

        for commit in &tagged_commits.commits {
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
                            if settings.include_unconventional {
                                ChangelogCommit::from_scm_commit(&commit);
                            }

                            return Err(ChangelogErrors::ChangelogError(e.to_string()));
                        }
                    };

                    changelog_entry_commits.push(c);
                }
                CommitStyleSettings::ReleaseNote(settings) => {
                    let release_notes = ReleaseNotes {
                        breaking_change_category: settings.breaking_change_category.clone(),
                    };
                    let release_notes = release_notes.parse_commit(&commit);
                    for release_note in release_notes {
                        changelog_entry_commits
                            .push(ChangelogCommit::from_release_note(&release_note));
                    }
                }
                CommitStyleSettings::Standard(_) => {
                    changelog_entry_commits.push(ChangelogCommit::from_scm_commit(&commit));
                }
            };

            'commits: for commit in changelog_entry_commits {
                if let Some(ignores) = &settings.commit.ignore {
                    for ignore in ignores {
                        if ignore.is_match(&commit)? {
                            if commit.breaking && protect_breaking {
                                debug!("Cant ignore commit {} as its a breaking change and protect_breaking_commits setting is enabled", &commit.id);
                            } else {
                                debug!("ignoring commit {}", &commit.id);
                                continue 'commits;
                            }
                        }
                    }
                }

                let entry = ChangelogEntry::new(
                    commit,
                    settings.commit.group_parsers.as_ref(),
                    settings.commit.link_parsers.as_ref(),
                )?;

                if entry.matched_group_parser || !filter_commits {
                    changelog_entries.push(entry);
                } else if filter_commits {
                    debug!(
                        "Skipping commit {} as it does not belong to any group",
                        &entry.id()
                    )
                }
            }
        }

        Ok(changelog_entries)
    }

    pub(crate) fn sort_releases(
        releases: &mut Vec<Release>,
        version_suffixes: Option<&Vec<String>>,
    ) {
        releases.sort_by(|a, b| {
            if b.version.is_some() && a.version.is_none() {
                return Ordering::Less;
            }

            if b.version.is_none() && a.version.is_some() {
                return Ordering::Greater;
            }

            if b.version.is_none() && a.version.is_none() {
                return Ordering::Equal;
            }

            let a_version = a.version.as_ref().unwrap();
            let b_version = b.version.as_ref().unwrap();

            let mut o = b_version
                .major()
                .cmp(&a_version.major())
                .then(b_version.minor().cmp(&a_version.minor()))
                .then(b_version.patch().cmp(&a_version.patch()));

            let a_modifier = a_version.modifier().unwrap_or_default();
            let b_modifier = b_version.modifier().unwrap_or_default();
            if let Some(version_suffixes) = &version_suffixes {
                let a_modifier_index = version_suffixes
                    .iter()
                    .position(|s| s.to_lowercase() == a_modifier.to_lowercase())
                    .unwrap_or(0);
                let b_modifier_index = version_suffixes
                    .iter()
                    .position(|s| s.to_lowercase() == b_modifier.to_lowercase())
                    .unwrap_or(0);
                o = o.then(b_modifier_index.cmp(&a_modifier_index));
            } else {
                o = o.then(b_modifier.cmp(&a_modifier));
            }

            o
        });
    }

    pub fn generate_individual<P: AsRef<Path>>(
        &self,
        path: P,
        format: MarkupFormat,
    ) -> ChangelogResult<()> {
        if path.as_ref().is_file() {
            // TODO: return error
        }

        fs::create_dir_all(&path)?;

        for release in &self.releases {
            let context = TemplateContext::from_serialize(release)?;

            let file_name = if let Some(version) = &release.version {
                format!("{}", version)
            } else {
                "unreleased.".to_string()
            };

            let changelog_path = path
                .as_ref()
                .join(file_name)
                .with_extension(format.extension());

            let mut f = File::options()
                .create(true)
                .append(true)
                .open(changelog_path)?;

            if let Some(header_template) = &self.header_template {
                let header = self.render(header_template, &context)?;
                writeln!(&mut f, "{}", header)?;
            }

            let body = self.render(&self.body_template, &context)?;
            writeln!(&mut f, "{}", body)?;

            if let Some(footer_template) = &self.footer_template {
                let footer = self.render(footer_template, &context)?;
                writeln!(&mut f, "{}", footer)?;
            }
        }

        Ok(())
    }

    pub fn generate<W: Write>(&self, out: &mut W) -> ChangelogResult<()> {
        // TODO: add additional context
        let context = TemplateContext::from_serialize(
            // &HashMap::from([("releases", &self.releases)])
            &Releases {
                releases: &self.releases,
            },
        )?;

        if let Some(header_template) = &self.header_template {
            let header = self.render(header_template, &context)?;
            writeln!(out, "{}", header)?;
        }

        let body = self.render(&self.body_template, &context)?;
        write!(out, "{}", body)?;

        if let Some(footer_template) = &self.footer_template {
            let footer = self.render(footer_template, &context)?;
            writeln!(out, "{}", footer)?;
        }

        Ok(())
    }

    fn render(&self, template: &str, context: &TemplateContext) -> ChangelogResult<String> {
        let mut rendered = Templates::one_off(template, &context, false)?;

        if self.trim {
            rendered = rendered.trim().to_string();
        }

        if let Some(post_processors) = &self.post_processors {
            for postprocessor in post_processors {
                postprocessor.replace(&mut rendered, vec![])?;
            }
        }

        Ok(rendered)
    }

    /// Generates a changelog and prepends it to the given changelog.
    pub fn prepend<W: Write>(&self, mut changelog: String, out: &mut W) -> ChangelogResult<()> {
        // TODO: this implementation has problems such as if the header changed.
        // I would like to go the AST route
        if let Some(header_template) = &self.header_template {
            let context = TemplateContext::from_serialize(&self.releases)?;
            let header = Templates::one_off(header_template, &context, false)?;
            changelog = changelog.replacen(&header, "", 1);
        }

        self.generate(out)?;
        write!(out, "{changelog}")?;
        Ok(())
    }

    // Increments the version for the unreleased changes
    pub fn bump_version(&self) {
        if let Some(mut release) = self.releases.first() {
            if let Some(version) = &release.version {}
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::path::PathBuf;

    use scm::commit::{ScmCommit, ScmSignature, ScmTag};
    use somever::{Somever, VersioningScheme};

    use super::Release;
    use crate::changelog::Changelog;
    use crate::commits::ScmTaggedCommits;
    use crate::settings::{ChangelogSettings, TemplateSettings};

    #[test]
    fn test_generator_groupby_repo() {
        let tagged_commits = vec![
            ScmTaggedCommits {
                repository: "lib".to_string(),
                tag: Option::from(ScmTag {
                    id: None,
                    name: "1.0.0".to_string(),
                    message: None,
                    timestamp: 0,
                }),
                commits: vec![ScmCommit {
                    id: "".to_string(),
                    message: "Added feature A".to_string(),
                    description: "".to_string(),
                    body: "".to_string(),
                    author: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    committer: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    timestamp: 0,
                }],
                timestamp: None,
            },
            ScmTaggedCommits {
                repository: "bin".to_string(),
                tag: Option::from(ScmTag {
                    id: None,
                    name: "1.0.0".to_string(),
                    message: None,
                    timestamp: 0,
                }),
                commits: vec![ScmCommit {
                    id: "".to_string(),
                    message: "Added feature A".to_string(),
                    description: "".to_string(),
                    body: "".to_string(),
                    author: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    committer: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    timestamp: 0,
                }],
                timestamp: None,
            },
        ];

        let settings = ChangelogSettings {
            output: Default::default(),
            template: TemplateSettings {
                body: r###"
{% if version -%}
    ## [{{ version }}] - {{ timestamp }}
{% else -%}
    ## [Unreleased]
{% endif -%}
{% for group, releases in releases|groupby("repository") -%}
    - {{ group }}
    {% for release in releases -%}
        {% for commit in release.commits -%}
        - {{ commit.message }}
        {% endfor %}
    {% endfor %}
{% endfor %}
"###
                .to_string(),
                ..Default::default()
            },
            commit: Default::default(),
            release: Default::default(),
            remote: None,
            bump: None,
            protect_breaking_commits: false,
            exclude_ungrouped: false,
            commit_version: None,
            version_scheme: Default::default(),
            version_suffixes: None,
        };

        let changelog = Changelog::new(tagged_commits, settings).unwrap();

        let mut output = File::create(PathBuf::from("./groupby_repo_changelog.md")).unwrap();
        changelog.generate(&mut output).unwrap()
    }

    #[test]
    fn test_generator_groupby_version() {
        let tagged_commits = vec![
            ScmTaggedCommits {
                repository: "lib".to_string(),
                tag: Option::from(ScmTag {
                    id: None,
                    name: "1.0.0".to_string(),
                    message: None,
                    timestamp: 0,
                }),
                commits: vec![ScmCommit {
                    id: "".to_string(),
                    message: "Added feature A".to_string(),
                    description: "".to_string(),
                    body: "".to_string(),
                    author: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    committer: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    timestamp: 1725164895,
                }],
                timestamp: None,
            },
            ScmTaggedCommits {
                repository: "bin".to_string(),
                tag: Option::from(ScmTag {
                    id: None,
                    name: "1.0.0".to_string(),
                    message: None,
                    timestamp: 0,
                }),
                commits: vec![ScmCommit {
                    id: "".to_string(),
                    message: "Added feature A".to_string(),
                    description: "".to_string(),
                    body: "".to_string(),
                    author: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    committer: ScmSignature {
                        name: None,
                        email: None,
                        timestamp: 0,
                    },
                    timestamp: 1725164895,
                }],
                timestamp: None,
            },
        ];

        let settings = ChangelogSettings {
            output: Default::default(),
            template: TemplateSettings {
                body: r###"
{% if version -%}
    ## [{{ version }}] - {{ timestamp | dateformat(format="%Y-%m-%d") }}
{% else -%}
    ## [Unreleased]
{% endif -%}
{% for group, releases in releases|groupby("version.value") -%}
    - {{ group }}
    {% for release in releases -%}
        {% for commit in release.commits -%}
        - {{ commit.message }} - {{ commit.timestamp|date(format="%Y-%m-%d") }}
        {% endfor %}
    {% endfor %}
{% endfor %}
"###
                .to_string(),
                ..Default::default()
            },
            commit: Default::default(),
            release: Default::default(),
            remote: None,
            bump: None,
            protect_breaking_commits: false,
            exclude_ungrouped: false,
            commit_version: None,
            version_scheme: Default::default(),
            version_suffixes: None,
        };

        let changelog = Changelog::new(tagged_commits, settings).unwrap();

        let mut output = File::create(PathBuf::from("./groupby_version_changelog.md")).unwrap();
        changelog.generate(&mut output).unwrap()
    }

    #[test]
    fn sort_releases() {
        let mut releases = vec![
            Release {
                version: Some(Somever::new(VersioningScheme::Semver, "1.0.0").unwrap()),
                tag_id: None,
                repository: "".to_string(),
                commits: vec![],
                timestamp: None,
            },
            Release {
                version: Some(Somever::new(VersioningScheme::Semver, "1.0.0-alpha").unwrap()),
                tag_id: None,
                repository: "".to_string(),
                commits: vec![],
                timestamp: None,
            },
            Release {
                version: Some(Somever::new(VersioningScheme::Semver, "1.0.0-final").unwrap()),
                tag_id: None,
                repository: "".to_string(),
                commits: vec![],
                timestamp: None,
            },
            Release {
                version: Some(Somever::new(VersioningScheme::Semver, "1.0.0-rc").unwrap()),
                tag_id: None,
                repository: "".to_string(),
                commits: vec![],
                timestamp: None,
            },
            Release {
                version: None,
                tag_id: None,
                repository: "".to_string(),
                commits: vec![],
                timestamp: None,
            },
        ];

        // TODO: would like these to include the prefix but not sure we can with semver implementation
        let version_suffixes = vec![
            "alpha".to_string(),
            "rc".to_string(),
            "".to_string(),
            "final".to_string(),
        ];

        Changelog::sort_releases(&mut releases, Some(&version_suffixes));
        println!("{:?}", releases);
        assert!(&releases[0].version.is_none());
        assert_eq!(
            "1.0.0-final".to_string(),
            releases[1].version.as_ref().unwrap().to_string()
        );
        assert_eq!(
            "1.0.0".to_string(),
            releases[2].version.as_ref().unwrap().to_string()
        );
        assert_eq!(
            "1.0.0-rc".to_string(),
            releases[3].version.as_ref().unwrap().to_string()
        );
        assert_eq!(
            "1.0.0-alpha".to_string(),
            releases[4].version.as_ref().unwrap().to_string()
        );
    }
}
