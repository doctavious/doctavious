use std::collections::HashMap;
use std::env;

use doctavious_std::command;
use doctavious_templating::Templates;
use regex::Regex;
use scm::drivers::git::TagSort;
use scm::providers::ScmProviders;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use somever::VersioningScheme;
use tracing::warn;

use crate::changelog::ChangelogKind;
use crate::entries::ChangelogCommit;
use crate::errors::{ChangelogErrors, ChangelogResult};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogSettings {
    // TODO: not sure about the name. Where should this go?
    pub structure: ChangelogKind,

    pub templates: TemplateSettings,

    pub scm: ChangelogScmSettings,
    pub remote: Option<ChangelogRemoteSettings>,
    pub bump: Option<ChangelogBumpSettings>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TemplateSettings {
    pub header: Option<String>,
    pub body: String,
    pub footer: Option<String>,
    pub trim: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogRemoteSettings {
    // TODO: flatten? ScmProvider lowercase?
    providers: HashMap<ScmProviders, ChangelogRemote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogRemote {
    /// Owner of the remote.
    pub owner: String,

    /// Repository name.
    pub repo: String,
    // TODO: how to handle secret token? put in keystore?
    // /// Access token.
    // #[serde(skip_serializing)]
    // pub token: Option<SecretString>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogBumpSettings {
    /// Configures automatic minor version increments for feature changes.
    ///
    /// When `true`, a feature will always trigger a minor version update.
    /// When `false`, a feature will trigger:
    ///
    /// - A patch version update if the major version is 0.
    /// - A minor version update otherwise.
    pub features_always_bump_minor: Option<bool>,

    /// Configures 0 -> 1 major version increments for breaking changes.
    ///
    /// When `true`, a breaking change commit will always trigger a major
    /// version update (including the transition from version 0 to 1)
    /// When `false`, a breaking change commit will trigger:
    ///
    /// - A minor version update if the major version is 0.
    /// - A major version update otherwise.
    pub breaking_always_bump_major: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConventionalCommitSettings {
    /// Whether to include unconventional commits.
    pub include_unconventional: Option<bool>,

    /// Whether to split commits by line, processing each line as an individual commit.
    pub split_commits: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReleaseNoteSettings {
    /// Category that flags commit as a breaking change.
    /// Defaults to breaking
    #[serde(default = "default_breaking_category")]
    pub breaking_change_category: String,
}

fn default_breaking_category() -> String {
    "breaking".to_string()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StandardCommitSettings {
    /// Whether to split commits by line, processing each line as an individual commit.
    pub split_commits: Option<bool>,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CommitStyleSettings {
    Conventional(ConventionalCommitSettings),
    ReleaseNote(ReleaseNoteSettings),
    Standard(StandardCommitSettings),
}

impl CommitStyleSettings {
    pub fn split_lines(&self) -> bool {
        match self {
            CommitStyleSettings::Conventional(s) => s.split_commits.unwrap_or_default(),
            CommitStyleSettings::ReleaseNote(_) => false,
            CommitStyleSettings::Standard(s) => s.split_commits.unwrap_or_default(),
        }
    }
}

impl Default for CommitStyleSettings {
    fn default() -> Self {
        Self::Standard(StandardCommitSettings::default())
    }
}

// TODO: dont really like the name
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogScmSettings {
    pub commit_version: Option<VersioningScheme>,

    pub commit_style: Option<CommitStyleSettings>,

    /// Commit preprocessors.
    pub commit_preprocessors: Option<Vec<CommitProcessor>>,

    pub skips: Option<Vec<CommitParser>>,

    /// Group parsers.
    pub group_parsers: Option<Vec<GroupParser>>,

    /// Link parsers.
    pub link_parsers: Option<Vec<LinkParser>>,

    /// Whether to protect all breaking changes from being skipped by a commit parser.
    pub protect_breaking_commits: Option<bool>,

    /// Whether to filter out commits.
    /// If set to true, commits that are not matched by group_parsers are filtered out.
    pub filter_commits: Option<bool>,

    /// Drop commits from the changelog
    #[serde(with = "serde_regex", default)]
    pub skip_tags: Option<Regex>,

    /// Regex to ignore matched tags.
    /// include ignored commits into the next tag.
    #[serde(with = "serde_regex", default)]
    pub ignore_tags: Option<Regex>,

    /// How to sort tags
    pub tag_sort: Option<TagSort>,

    // TODO: use enum? oldest / newest
    /// Sorting of the commits inside sections.
    pub sort_commits: Option<String>,

    /// Limit the number of commits included in the changelog.
    pub limit_commits: Option<usize>,

    pub version_scheme: VersioningScheme,

    /// determine the sorting order of tags with different suffixes
    /// The placement of the main release tag relative to tags with various suffixes can be
    /// determined by specifying the empty suffix among those other suffixes.
    pub version_suffixes: Option<Vec<String>>,
}

/// Parser for grouping commits.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitParser {
    /// Field name of the commit to match the regex against.
    pub field: String,

    /// Regex for matching the field value.
    #[serde(with = "serde_regex")]
    pub pattern: Regex,
}

impl CommitParser {
    pub fn is_match(&self, commit: &ChangelogCommit) -> ChangelogResult<bool> {
        Ok(self.matched(commit)?.is_some())
    }

    pub fn dot_path<'a>(value: &'a Value, path: &'a str) -> Option<&'a Value> {
        path.split('.')
            .try_fold(value, |target, token| match target {
                Value::Object(map) => map.get(token),
                _ => None,
            })
    }

    pub fn matched(&self, commit: &ChangelogCommit) -> ChangelogResult<Option<String>> {
        let mut checks = Vec::new();
        // TODO: need to support multiple field checks
        // { message = ".*deprecated", body = ".*deprecated", group = "Deprecation" }
        // TODO: support entire entry / dot path notation
        let field = self.field.as_str();
        let pattern = &self.pattern;

        let commit_value = serde_json::to_value(commit)?;
        let value = Self::dot_path(&commit_value, field);
        if let Some(v) = value {
            match v {
                Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                    checks.push(Some(v.to_string()))
                }
                Value::Array(a) => checks.extend(a.iter().map(|f| Some(f.to_string()))),
                Value::Null => {
                    warn!("skipping commit parser with field {field} as it has no value");
                }
                Value::Object(_) => {
                    warn!("skipping commit parser with field {field} as value is not a scalar or array");
                }
            }
        }

        if checks.is_empty() {
            return Err(ChangelogErrors::ChangelogError(format!(
                "invalid group parser field {field}",
            )));
        }

        for text in checks {
            if let Some(text) = text {
                if pattern.is_match(&text) {
                    return Ok(Some(text));
                }
            } else {
                warn!("Skip group parser for field {field} does not have a value")
            }
        }

        Ok(None)
    }
}

/// Parser for grouping commits.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupParser {
    // /// Field name of the commit to match the regex against.
    // pub field: String,
    //
    // /// Regex for matching the field value.
    // #[serde(with = "serde_regex")]
    // pub pattern: Regex,

    // TODO: support multiple
    // TODO: is 'rules' a better name?
    #[serde(flatten)]
    pub commit_parser: CommitParser,

    // #[serde(flatten)]
    // pub commit_parser: CommitParser,
    /// Group of the commit.
    pub group: Option<String>,

    /// Default scope of the commit.
    pub default_scope: Option<String>,

    /// Commit scope for overriding the default scope.
    pub scope: Option<String>,

    /// Whether to skip this commit group.
    pub skip: Option<bool>,
}

// Manipulating commit messages before grouping.
// These regex-based preprocessors can be used for removing or selecting certain parts of the
// commit message/body to be used in the following processes.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommitProcessor {
    /// Regex for matching a text to replace.
    #[serde(with = "serde_regex")]
    pub pattern: Regex,

    /// Replacement text.
    pub replace: Option<String>,

    /// Command that will be run for replacing the commit message.
    pub replace_command: Option<String>,
}

impl CommitProcessor {
    /// Replaces the text with using the given pattern or the command output.
    pub fn replace(
        &self,
        rendered: &mut String,
        command_envs: Vec<(&str, &str)>,
    ) -> ChangelogResult<()> {
        if let Some(text) = &self.replace {
            *rendered = self.pattern.replace_all(rendered, text).to_string();
        } else if let Some(command) = &self.replace_command {
            if self.pattern.is_match(rendered) {
                *rendered = command::run(
                    command,
                    Some(rendered.to_string()),
                    &env::current_dir()?,
                    command_envs,
                )
                .map_err(|e| ChangelogErrors::CommitParser)?;
            }
        }
        Ok(())
    }
}

/// Parser for extracting links in commits.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LinkParser {
    /// Regex for finding links in the commit message.
    #[serde(with = "serde_regex")]
    pub pattern: Regex,

    /// The string used to generate the link URL.
    pub href: String,

    /// The string used to generate the link text.
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_derive::Serialize;

    use crate::settings::CommitParser;

    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct Inner {
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Serialize)]
    struct Outer {
        body: String,
        inner: Inner,
    }

    // TODO: invalid template should return valid error
    #[test]
    fn c() {
        let o = Outer {
            body: "some message".to_string(),
            inner: Inner {
                name: "sean".to_string(),
            },
        };

        let value = serde_json::to_value(o).unwrap();

        let r = CommitParser::dot_path(&value, "body");
        println!("{:?}", r);
    }
}
