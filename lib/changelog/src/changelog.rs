use std::io::Write;

use git_conventional::{Commit as GitConventionalCommit, Error};
use scm::commit::{ScmCommit, TaggedCommits};
use strum::IntoEnumIterator;
use tracing::{trace, warn};
use doctavious_templating::{TemplateContext, Templates};
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
                            // Ok(conv) => ChangelogCommit::Conventional(ConventionalCommit {
                            //     commit: commit.clone(),
                            //     conv,
                            // }),
                            Err(e) => {
                                if settings.include_unconventional.unwrap_or_default() {
                                    // ChangelogCommit::Standard(commit.clone());
                                    ChangelogCommit::from_scm_commit(&commit);
                                }

                                return Err(ChangelogErrors::ChangelogError("".to_string()));
                            }
                        };

                        changelog_entry_commits.push(c);
                    }
                    CommitStyleSettings::ReleaseNote(settings) => {
                        let release_notes = ReleaseNote::parse_commit(&commit);
                        for release_note in release_notes {
                            // changelog_entry_commits.push(ChangelogCommit::ReleaseNote(release_note));
                            changelog_entry_commits.push(
                                ChangelogCommit::from_release_note(&release_note)
                            );
                        }
                    }

                    CommitStyleSettings::Standard(_) => {
                       // changelog_entry_commits.push(ChangelogCommit::Standard(commit.clone()));
                        changelog_entry_commits.push(ChangelogCommit::from_scm_commit(&commit));
                    }
                };

                for c in changelog_entry_commits {
                    if let Some(skips) = &settings.scm.skips {
                        for skip in skips {}
                    }

                    let mut entry = ChangelogEntry::new(c);
                    if entry.matched_group_parser || !filter_commits {
                        changelog_entries.push(entry);
                    } else if filter_commits {
                        trace!("Commit {} does not belong to any group", &entry.id())
                    }
                }

                // for c in changelog_entry_commits {
                //     let entry = ChangelogEntry {
                //         id: "".to_string(),
                //         message: "".to_string(),
                //         commit: c,
                //         group: None,
                //         default_scope: None,
                //         scope: None,
                //         timestamp: 0,
                //         links: vec![],
                //         author: Default::default(),
                //         committer: Default::default(),
                //         merge_commit: false,
                //     };
                // }

                // // Sort Oldest - lists newest first to oldest
                // if options.sort == ChangelogCommitSort::Newest {
                //     releases[release_index].commits.insert(0, commit);
                // } else {
                //     releases[release_index].commits.push(commit);
                // }
                //
                // if let Some(tag) = tags.get(&commit_id) {
                //     releases[release_index].version = Some(tag.to_string());
                //     releases[release_index].commit_id = Some(commit_id);
                //     releases[release_index].timestamp = if options.tag.as_deref() == Some(tag) {
                //         SystemTime::now()
                //             .duration_since(UNIX_EPOCH)?
                //             .as_secs()
                //             .try_into()?
                //     } else {
                //         scm_commit.timestamp
                //     };
                //
                //     releases.push(Release::default());
                //     release_index += 1;
                // }

                // if let Some(group_parsers) = &settings.scm.group_parsers {
                //     for group_parser in group_parsers {
                //         let field = group_parser.field.as_ref();
                //         let pattern = &group_parser.pattern;
                //         let text = match field {
                //             "id" => Some(commit.id.clone()),
                //             "message" => Some(commit.message.clone()),
                //             "body" => Some(commit.message.clone()),
                //             "author.name" => commit.author.name.clone(),
                //             "author.email" => commit.author.email.clone(),
                //             "committer.name" => commit.committer.name.clone(),
                //             "committer.email" => commit.committer.email.clone(),
                //             _ => None,
                //         }
                //         .ok_or_else(|| {
                //             ChangelogErrors::ChangelogError(format!(
                //                 "field {} does not have a value",
                //                 field
                //             ))
                //         })?;
                //
                //         if pattern.is_match("") {}
                //     }
                // }
                //
                // if let Some(parsers) = &settings.scm.link_parsers {
                //     let mut links = vec![];
                //     for parser in parsers {
                //         let regex = &parser.pattern;
                //         let replace = &parser.href;
                //         for mat in regex.find_iter(&commit.message) {
                //             let m = mat.as_str();
                //             let text = if let Some(text_replace) = &parser.text {
                //                 regex.replace(m, text_replace).to_string()
                //             } else {
                //                 m.to_string()
                //             };
                //             let href = regex.replace(m, replace);
                //             links.push(Link {
                //                 text,
                //                 href: href.to_string(),
                //             });
                //         }
                //     }
                // }
            }

            let tag= tagged_commit.tag;

            // if args.tag == Some(tag_name.clone())
            // {
            //     SystemTime::now()
            //         .duration_since(UNIX_EPOCH)?
            //         .as_secs()
            //         .try_into()?
            // } else {
            //     git_commit.time().seconds()
            // };

            // TODO: transform ScmTag to SomeVer based on settings
            releases.push(Release {
                version: tag.as_ref().map(|t| t.name.clone()),
                tag_id: tag.as_ref().map(|t| t.id.clone()).unwrap_or_default(),
                commits: changelog_entries,
                timestamp: 0,
            });
        }

        // TODO: sort releases based on settings

        Ok(Self {
            releases,
        })
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

    // fn parse(
    //     // commit: &ChangelogCommit,
    //     entry: &mut ChangelogEntry,
    //     parsers: &[GroupParser],
    //     protect_breaking: bool,
    //     filter: bool,
    // ) {
    //     for group_parser in parsers {
    //         let field = group_parser.field.as_ref();
    //         let pattern = &group_parser.pattern;
    //         let text = match field {
    //             "id" => Some(entry.id()),
    //             "message" => Some(entry.message()),
    //             "description" => Some(entry.description()),
    //             "body" => Some(entry.body()),
    //             "footer" => Some(entry.footers().iter().map(|f| f.to_string())),
    //             "author.name" => entry.author().name.as_deref(),
    //             "author.email" => entry.author().email.as_deref(),
    //             "committer.name" => entry.committer().name.as_deref(),
    //             "committer.email" => entry.committer().email.as_deref(),
    //             _ => None,
    //         }.ok_or_else(|| {
    //             ChangelogErrors::ChangelogError(format!(
    //                 "field {} does not have a value",
    //                 field
    //             ))
    //         }).unwrap();
    //
    //         if pattern.is_match(text) {
    //             let regex_replace = |mut value: String| {
    //                 for mat in pattern.find_iter(&text) {
    //                     value = pattern.replace(mat.as_str(), value).to_string();
    //                 }
    //                 value
    //             };
    //
    //             entry.group = group_parser.group.as_ref().cloned().map(regex_replace);
    //             entry.scope = group_parser.scope.as_ref().cloned().map(regex_replace);
    //             entry.default_scope = group_parser.default_scope.as_ref().cloned();
    //             return;
    //         }
    //     }
    // }

    fn parse_links(commit: &ChangelogCommit, parsers: &[LinkParser]) -> Vec<Link> {
        let mut links = vec![];
        // for parser in parsers {
        //     let regex = &parser.pattern;
        //     let replace = &parser.href;
        //     for mat in regex.find_iter(&commit.message) {
        //         let m = mat.as_str();
        //         let text = if let Some(text_replace) = &parser.text {
        //             regex.replace(m, text_replace).to_string()
        //         } else {
        //             m.to_string()
        //         };
        //         let href = regex.replace(m, replace);
        //         links.push(Link {
        //             text,
        //             href: href.to_string(),
        //         });
        //     }
        // }

        links
    }

    // fn new(releases: Vec<Release<'a>>, settings: ChangelogScmSettings) -> ChangelogResult<Self> {
    //     let mut changelong = Self { releases };
    //     changelong.process_commits(&settings);
    //
    //     Ok(changelong)
    // }
    //
    // // TODO: process_commits
    // fn process_commits(&mut self, settings: &ChangelogScmSettings) {
    //     for release in self.releases.iter() {
    //         for commit in release.commits.iter() {
    //             match self.process_commit(commit, settings) {
    //                 Ok(_) => {}
    //                 Err(_) => {}
    //             }
    //         }
    //     }
    //
    //     // self.releases.iter_mut().for_each(|release| {
    //     //     release.commits = release
    //     //         .commits
    //     //         .iter()
    //     //         .cloned()
    //     //         .filter_map(|commit| {
    //     //             // TODO: not yet sure if it makes sense for process to be on the commit
    //     //             // match commit.process() {
    //     //             //     Ok(_) => {}
    //     //             //     Err(_) => {}
    //     //             // }
    //     //             match self.process_commit(&commit) {
    //     //                 Ok(_) => {}
    //     //                 Err(_) => {}
    //     //             }
    //     //             None
    //     //         })
    //     //         .flat_map(|commit| {
    //     //             // if self.config.git.split_commits.unwrap_or(false) {
    //     //             //     commit
    //     //             //         .message
    //     //             //         .lines()
    //     //             //         .filter_map(|line| {
    //     //             //             let mut c = commit.clone();
    //     //             //             c.message = line.to_string();
    //     //             //             if !c.message.is_empty() {
    //     //             //                 Self::process_commit(c, &self.config.git)
    //     //             //             } else {
    //     //             //                 None
    //     //             //             }
    //     //             //         })
    //     //             //         .collect()
    //     //             // } else {
    //     //             //     vec![commit]
    //     //             // }
    //     //             vec![commit]
    //     //         })
    //     //         .collect::<Vec<Commit>>();
    //     // });
    // }

    // fn process_commit(
    //     &self,
    //     commit: &ScmCommit,
    //     settings: &ChangelogSettings,
    // ) -> ChangelogResult<()> {
    //     // TODO: preprocessors - An array of commit preprocessors for manipulating the commit messages before parsing/grouping them.
    //     // These regex-based preprocessors can be used for removing or selecting certain parts of the commit message/body to be used
    //     // in the following processes.
    //
    //     if settings.split_commits.unwrap_or(false) {
    //         if let Some(message) = &commit.message {
    //             let lines: Vec<_> = message
    //                 .lines()
    //                 .filter_map(|line| {
    //                     if line.is_empty() {
    //                         None
    //                     } else {
    //                         Some(line.to_string())
    //                     }
    //                 })
    //                 .collect();
    //         }
    //     }
    //
    //     if let Some(preprocessors) = &settings.commit_preprocessors {
    //         for preprocessor in preprocessors {
    //             // preprocessor.replace(&mut commit.message, vec![]);
    //         }
    //     }
    //
    //     if settings.conventional_commits.unwrap_or(true) {}
    //
    //     if let Some(group_parsers) = &settings.group_parsers {
    //         for group_parser in group_parsers {
    //             let field = group_parser.field.as_ref();
    //             let pattern = &group_parser.pattern;
    //             let text = match field {
    //                 "id" => Some(commit.id.clone()),
    //                 "message" => commit.message.clone(),
    //                 "body" => commit.message.clone(),
    //                 "author.name" => commit.author.name.clone(),
    //                 "author.email" => commit.author.email.clone(),
    //                 "committer.name" => commit.committer.name.clone(),
    //                 "committer.email" => commit.committer.email.clone(),
    //                 _ => None,
    //             }
    //             .ok_or_else(|| {
    //                 ChangelogErrors::ChangelogError(format!(
    //                     "field {} does not have a value",
    //                     field
    //                 ))
    //             })?;
    //
    //             if pattern.is_match("") {}
    //         }
    //     }
    //
    //     if let Some(parsers) = &settings.link_parsers {
    //         let mut links = vec![];
    //         for parser in parsers {
    //             let regex = &parser.pattern;
    //             let replace = &parser.href;
    //             if let Some(message) = &commit.message {
    //                 for mat in regex.find_iter(message) {
    //                     let m = mat.as_str();
    //                     let text = if let Some(text_replace) = &parser.text {
    //                         regex.replace(m, text_replace).to_string()
    //                     } else {
    //                         m.to_string()
    //                     };
    //                     let href = regex.replace(m, replace);
    //                     links.push(Link {
    //                         text,
    //                         href: href.to_string(),
    //                     });
    //                 }
    //             }
    //         }
    //     }
    //
    //     // TODO: split commits
    //     // TODO: convention commit / release note / standard commit
    //     // TODO: commit parsers - An array of commit parsers for determining the commit groups by using regex.
    //     // TODO: link parsers - An array of link parsers for extracting external references, and turning them into URLs, using regex.
    //
    //     if let Some(message) = &commit.message {
    //         if let Ok(conventional_commit) = ConventionalCommit::parse(message) {
    //             println!(
    //                 "{}",
    //                 format!(
    //                     "{} - {} with scope {:?}",
    //                     &commit.timestamp,
    //                     &commit.message,
    //                     &conventional_commit.scope()
    //                 )
    //             );
    //         } else {
    //             println!("{}", format!("{} - {}", &commit.timestamp, &commit.message));
    //         }
    //     }
    //
    //     Ok(())
    // }

    // TODO: process_releases

    // TODO: generate
    pub fn generate<W: Write>(&self, out: &mut W) -> ChangelogResult<()> {
        // TODO: post processors

        // TODO: headers

        println!("rendering...");
        // TODO: support versions in different files
        for release in &self.releases {

            let template =
r###"
{% if version -%}
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else -%}
    ## [Unreleased]
{% endif -%}
{% for commit in commits -%}
    - {{ commit.message }}
{% endfor %}
"###;
            let context = TemplateContext::from_serialize(release).unwrap();
            let rendered = Templates::one_off(template, context, false)?;
            write!(
                out,
                "{}",
                rendered
            )?;
        }

        // TODO: footers

        Ok(())
    }

    // TODO: prepend

    pub fn bump_version(&self) {}
}


#[cfg(test)]
mod test {

    #[test]
    fn changelog_generator() {

    }

}