use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use doctavious_std::command;
use doctavious_templating::Templates;
use glob::Pattern;
use markup::MarkupFormat;
use regex::Regex;
use scm::drivers::git::TagSort;
use scm::providers::ScmProviders;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use somever::VersioningScheme;
use strum::{Display, EnumIter, EnumString, VariantNames};
use tracing::warn;

use crate::changelog::{Changelog, ChangelogOutputType};
use crate::entries::ChangelogCommit;
use crate::errors::{ChangelogErrors, ChangelogResult};

pub struct ChangelogConfigurationFile {}

pub struct ChangelogConfiguration {
    // pub range: Option<ChangelogRange>,
    // pub include_paths: Option<Vec<Pattern>>,
    // pub exclude_paths: Option<Vec<Pattern>>,
    // pub commit_sort: ChangelogCommitSort,

    // #[serde(flatten)]
    // pub output: ChangelogOutput,
    // pub templates: TemplateSettings,
    // pub scm: ChangelogScmSettings,
    // pub remote: Option<ChangelogRemoteSettings>,
    // pub bump: Option<ChangelogBumpSettings>,
}

// TODO: rename to ChangelogConfiguration?
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogSettings {
    #[serde(flatten)]
    pub output: ChangelogOutput,
    pub template: TemplateSettings,
    pub commit: ChangelogCommitSettings,
    pub release: ChangelogReleaseConfiguration,
    pub remote: Option<ChangelogRemoteSettings>,
    pub bump: Option<ChangelogBumpSettings>,

    /// Whether to protect all breaking changes from being skipped by a commit parser.
    pub protect_breaking_commits: bool,

    /// Whether to exclude entries that do not belong to any group from the changelog.
    pub exclude_ungrouped: bool,

    pub commit_version: Option<VersioningScheme>,
    pub version_scheme: VersioningScheme,
    /// determine the sorting order of tags with different suffixes
    /// The placement of the main release tag relative to tags with various suffixes can be
    /// determined by specifying the empty suffix among those other suffixes.
    pub version_suffixes: Option<Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogReleaseConfiguration {
    pub tag_patterns: Option<Vec<String>>,

    /// Regexes to skip matched tags.
    /// include skipped tagged commits into the next tag.
    pub skip_tags: Option<Vec<String>>,

    /// Drop commits from the changelog
    pub ignore_tags: Option<Vec<String>>,

    /// How to sort tags
    pub tag_sort: Option<TagSort>,
}

#[non_exhaustive]
#[remain::sorted]
#[derive(Clone, Debug, Display, Deserialize, EnumString, Serialize, VariantNames)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum ChangelogOutput {
    Individual(IndividualChangelogOutput),
    Single(SingleChangelogOutput),
}

impl Default for ChangelogOutput {
    fn default() -> Self {
        Self::Single(SingleChangelogOutput::default())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IndividualChangelogOutput {
    path: PathBuf,
    format: MarkupFormat,
}

impl Default for IndividualChangelogOutput {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./changelogs/"),
            format: MarkupFormat::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SingleChangelogOutput {
    path: PathBuf,
    prepend: Option<PathBuf>,
}

impl Default for SingleChangelogOutput {
    fn default() -> Self {
        Self {
            path: PathBuf::from("changelog.md"),
            prepend: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TemplateSettings {
    /// A template to be rendered as the changelog's header.
    pub header: Option<String>,

    /// A template to be rendered for changelog's releases.
    pub body: String,

    /// A template to be rendered as the changelog's footer.
    pub footer: Option<String>,

    // TODO: git-cliff defaults this to true when not provided
    //       should we instead flip to "preserve_whitespace"?
    /// Whether to remove leading and trailing whitespaces from all lines of the changelog's templates.
    pub trim: bool,

    /// A list of postprocessors using regex to modify the changelog.
    pub post_processors: Option<Vec<CommitProcessor>>,
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
    pub features_always_bump_minor: bool,

    /// Configures 0 -> 1 major version increments for breaking changes.
    ///
    /// When `true`, a breaking change commit will always trigger a major
    /// version update (including the transition from version 0 to 1)
    /// When `false`, a breaking change commit will trigger:
    ///
    /// - A minor version update if the major version is 0.
    /// - A major version update otherwise.
    pub breaking_always_bump_major: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConventionalCommitSettings {
    /// Whether to include unconventional commits.
    pub include_unconventional: bool,

    /// Whether to split commits by line, processing each line as an individual commit.
    pub split_commits: bool,
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
    pub split_commits: bool,
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
            CommitStyleSettings::Conventional(s) => s.split_commits,
            CommitStyleSettings::ReleaseNote(_) => false,
            CommitStyleSettings::Standard(s) => s.split_commits,
        }
    }
}

impl Default for CommitStyleSettings {
    fn default() -> Self {
        Self::Standard(StandardCommitSettings::default())
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogCommitSettings {
    pub commit_style: CommitStyleSettings,

    /// Commit preprocessors.
    pub commit_preprocessors: Option<Vec<CommitProcessor>>,

    // Commits to ignore
    pub ignore: Option<Vec<CommitParser>>,

    /// Group parsers.
    pub group_parsers: Option<Vec<GroupParser>>,

    /// Link parsers.
    pub link_parsers: Option<Vec<LinkParser>>,

    /// Sorting of the commits inside sections.
    pub sort_commits: Option<ChangelogCommitSort>,

    /// Limit the number of commits included in the changelog.
    pub limit_commits: Option<usize>,
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
                    warn!(
                        "skipping commit parser with field {field} as value is not a scalar or array"
                    );
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

#[remain::sorted]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    Deserialize,
    EnumIter,
    EnumString,
    VariantNames,
    PartialEq,
    Serialize,
)]
pub enum ChangelogCommitSort {
    /// Whether to sort starting with the newest element.
    NewestFirst,

    /// Whether to sort starting with the oldest element.
    #[default]
    OldestFirst,
}

impl ChangelogCommitSort {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }
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
