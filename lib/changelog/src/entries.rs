use std::borrow::Borrow;
use std::fmt;
use std::fmt::{Display, Formatter};

use git2::Signature as CommitSignature;
use git_conventional::{Footer as ConventionalFooter};
use serde::Serialize;

use scm::commit::{ScmCommit, ScmSignature};

use crate::conventional::ConventionalCommit;
use crate::errors::ChangelogResult;
use crate::release_notes::ReleaseNote;
use crate::settings::{GroupParser, LinkParser};



// Initially had the following structure
// #[derive(Debug, Serialize)]
// pub enum ChangelogCommit<'a> {
//     Standard(ScmCommit),
//     Conventional(ConventionalCommit<'a>),
//     ReleaseNote(ReleaseNote),
// }
// however git_conventional's Commit has causing lifetime issues as the SCM commit that is parsed
// doesn't live as long as the change log entry. As a result I opted have struct that contains all
// necessary fields and copy them from the appropriate sources
#[derive(Debug, Serialize)]
pub struct ChangelogCommit {
    pub id: String,
    pub message: String,
    pub description: String,
    pub body: String,
    pub footers: Option<Vec<Footer>>,
    pub timestamp: i64,
    pub commit_style: String,
    pub commit_type: Option<String>,
    pub scope: Option<String>,
    pub author: ScmSignature,
    pub committer: ScmSignature
}

impl ChangelogCommit {
    pub fn from_scm_commit(commit: &ScmCommit) -> Self {
        Self {
            id: commit.id.to_string(),
            message: commit.message.to_string(),
            description: commit.description.to_string(),
            body: commit.body.to_string(),
            footers: None, // TODO: parse footers for ScmCommit
            timestamp: commit.timestamp,
            commit_style: "".to_string(),
            commit_type: None,
            scope: None,
            author: commit.author.clone(),
            committer: commit.committer.clone(),
        }
    }

    pub fn from_release_note(release_note: &ReleaseNote) -> Self {
        Self {
            id: release_note.commit.id.to_string(),
            message: release_note.commit.message.to_string(),
            description: release_note.commit.description.to_string(),
            body: release_note.commit.body.to_string(),
            footers: None, // TODO: parse footers for ScmCommit
            timestamp: release_note.commit.timestamp,
            commit_style: "".to_string(),
            commit_type: None,
            scope: None,
            author: release_note.commit.author.clone(),
            committer: release_note.commit.committer.clone(),
        }
    }

    pub fn from_conventional(conventional: ConventionalCommit) -> Self {
        Self {
            id: conventional.commit.id.to_string(),
            message: conventional.commit.message.to_string(),
            description: conventional.commit.description.to_string(),
            body: conventional.commit.body.to_string(),
            footers: Some(conventional.conv.footers().iter().map(Footer::from).collect::<Vec<Footer>>()),
            timestamp: conventional.commit.timestamp,
            commit_style: "".to_string(),
            commit_type: Some(conventional.conv.type_().to_string()),
            scope: conventional.conv.scope().map(|scope| scope.to_string()),
            author: conventional.commit.author.clone(),
            committer: conventional.commit.committer.clone(),
        }
    }
}

// #[derive(Debug, Serialize)]
// pub enum ChangelogCommit<'a> {
//     Standard(ScmCommit),
//     Conventional(ConventionalCommit<'a>),
//     ReleaseNote(ReleaseNote),
// }
//
// impl<'a> ChangelogCommit<'a> {
//     pub fn id(&self) -> &str {
//         match &self {
//             ChangelogCommit::Standard(c) => c.id.as_str(),
//             ChangelogCommit::Conventional(c) => c.commit.id.as_str(),
//             ChangelogCommit::ReleaseNote(c) => c.commit.id.as_str(),
//         }
//     }
//
//     pub fn message(&self) -> &str {
//         match &self {
//             ChangelogCommit::Standard(c) => c.message.as_str(),
//             ChangelogCommit::Conventional(c) => c.commit.message.as_str(),
//             ChangelogCommit::ReleaseNote(c) => c.commit.message.as_str(),
//         }
//     }
//
//     pub fn description(&self) -> &str {
//         match &self {
//             ChangelogCommit::Standard(c) => c.description.as_str(),
//             ChangelogCommit::Conventional(c) => c.commit.description.as_str(),
//             ChangelogCommit::ReleaseNote(c) => c.commit.description.as_str(),
//         }
//     }
//
//     pub fn body(&self) -> &str {
//         match &self {
//             ChangelogCommit::Standard(c) => c.body.as_str(),
//             ChangelogCommit::Conventional(c) => c.commit.body.as_str(),
//             ChangelogCommit::ReleaseNote(c) => c.commit.body.as_str(),
//         }
//     }
//
//     pub fn footers(&self) -> Option<Vec<Footer>> {
//         // TODO: parse footers for ScmCommit
//         match &self {
//             ChangelogCommit::Standard(_) => None,
//             ChangelogCommit::Conventional(c) => {
//                 Some(c.conv.footers().iter().map(Footer::from).collect())
//             }
//             ChangelogCommit::ReleaseNote(_) => None,
//         }
//     }
//
//     pub fn timestamp(&self) -> i64 {
//         match &self {
//             ChangelogCommit::Standard(c) => c.timestamp,
//             ChangelogCommit::Conventional(c) => c.commit.timestamp,
//             ChangelogCommit::ReleaseNote(c) => c.commit.timestamp,
//         }
//     }
//
//     pub fn commit_type(&self) -> Option<&str> {
//         match &self {
//             ChangelogCommit::Standard(_) => None,
//             ChangelogCommit::Conventional(c) => Some(c.conv.type_().as_str()),
//             ChangelogCommit::ReleaseNote(_) => None,
//         }
//     }
//
//     pub fn scope(&self) -> Option<&str> {
//         match &self {
//             ChangelogCommit::Standard(_) => None,
//             ChangelogCommit::Conventional(c) => c.conv.scope().map(|s| s.as_str()),
//             ChangelogCommit::ReleaseNote(r) => r.category.as_ref().map(|s| s.as_str()),
//         }
//     }
//
//     pub fn author(&self) -> &ScmSignature {
//         match &self {
//             ChangelogCommit::Standard(c) => c.author.borrow(),
//             ChangelogCommit::Conventional(c) => c.commit.author.borrow(),
//             ChangelogCommit::ReleaseNote(c) => c.commit.author.borrow(),
//         }
//     }
//
//     pub fn committer(&self) -> &ScmSignature {
//         match &self {
//             ChangelogCommit::Standard(c) => c.committer.borrow(),
//             ChangelogCommit::Conventional(c) => c.commit.committer.borrow(),
//             ChangelogCommit::ReleaseNote(c) => c.commit.committer.borrow(),
//         }
//     }
//
// }

#[derive(Debug, Serialize)]
pub struct ChangelogEntry {
    // /// Commit ID.
    // pub id: String,
    //
    // /// Commit message including title, description and summary.
    // pub message: String,
    #[serde(flatten)]
    pub commit: ChangelogCommit,

    /// Commit group based on a group parser or its conventional type.
    pub group: Option<String>,

    /// Default commit scope based on (inherited from) conventional type or a group parser.
    pub default_scope: Option<String>,

    /// Commit scope for overriding the default one.
    pub scope: Option<String>,

    pub matched_group_parser: bool,

    // pub timestamp: i64,
    /// A list of links found in the commit
    pub links: Vec<Link>,
    // /// Commit author.
    // pub author: Signature,
    //
    // /// Committer.
    // pub committer: Signature,
    //
    // /// Whether if the commit has two or more parents.
    // pub merge_commit: bool,
}

pub struct GroupParsing {
    parsers: Vec<GroupParser>,
    protect_breaking: bool,
    filter: bool,
}

struct ChangelogGroup {
    /// Commit group based on a group parser or its conventional type.
    pub group: Option<String>,

    /// Commit scope for overriding the default one.
    pub scope: Option<String>,

    /// Default commit scope based on (inherited from) conventional type or a group parser.
    pub default_scope: Option<String>,
}

impl ChangelogEntry {
    pub fn n(
        commit: ChangelogCommit,
        group_parsers: Option<&[GroupParser]>,
        protect_breaking: bool,
        filter: bool,
        link_parsers: Option<&[LinkParser]>,
    ) -> ChangelogResult<Self> {
        // TODO: clean up a bit around matched
        let (group, scope, default_scope, matched) = if let Some(group_parsers) = group_parsers {
            if let Some(group) = ChangelogEntry::determine_group(&commit, group_parsers)? {
                let ChangelogGroup {
                    group,
                    scope,
                    default_scope,
                } = group;
                (group, scope, default_scope, true)
            } else {
                (None, None, None, false)
            }
        } else {
            (None, None, None, false)
        };

        let links = if let Some(link_parsers) = link_parsers {
            ChangelogEntry::extract_links(&commit, link_parsers)
        } else {
            vec![]
        };

        Ok(Self {
            commit,
            group,
            scope, //: scope.or_else(|| commit.scope().map(|s| s.to_string())).or(default_scope),
            default_scope,
            matched_group_parser: matched,
            links,
        })
    }

    fn determine_group(
        commit: &ChangelogCommit,
        parsers: &[GroupParser],
    ) -> ChangelogResult<Option<ChangelogGroup>> {
        for group_parser in parsers {
            let matched = &group_parser.commit_parser.matched(commit)?;
            if let Some(text) = matched {
                let pattern = &group_parser.commit_parser.pattern;
                let regex_replace = |mut value: String| {
                    for mat in group_parser.commit_parser.pattern.find_iter(&text) {
                        value = pattern.replace(mat.as_str(), value).to_string();
                    }
                    value
                };

                return Ok(Some(ChangelogGroup {
                    group: group_parser.group.as_ref().cloned().map(regex_replace),
                    scope: group_parser.scope.as_ref().cloned().map(regex_replace),
                    default_scope: group_parser.default_scope.as_ref().cloned(),
                }));
            }


            // let mut checks = Vec::new();
            //
            // // TODO: need to support multiple field checks
            // // { message = ".*deprecated", body = ".*deprecated", group = "Deprecation" }
            // // TODO: support entire entry / dot path notation
            // let field = parser.field.as_ref();
            // let pattern = &parser.pattern;
            // match field {
            //     "id" => checks.push(Some(commit.id().to_string())),
            //     "message" => checks.push(Some(commit.message().to_string())),
            //     "description" => checks.push(Some(commit.description().to_string())),
            //     "body" => checks.push(Some(commit.body().to_string())),
            //     "footer" => {
            //         if let Some(footers) = commit.footers() {
            //             checks.extend(footers.iter().map(|f| Some(f.to_string())))
            //         }
            //     }
            //     "author.name" => checks.push(commit.author().name.clone()),
            //     "author.email" => checks.push(commit.author().email.clone()),
            //     "committer.name" => checks.push(commit.committer().name.clone()),
            //     "committer.email" => checks.push(commit.committer().email.clone()),
            //     _ => {}
            // };
            //
            // if checks.is_empty() {
            //     return Err(ChangelogErrors::ChangelogError(format!(
            //         "invalid group parser field {field}",
            //     )));
            // }
            //
            // for text in checks {
            //     if let Some(text) = text {
            //         if pattern.is_match(&text) {
            //             // TODO: test for the following
            //             // { message = '^fix\((.*)\)', group = 'Fix (${1})' }
            //             let regex_replace = |mut value: String| {
            //                 for mat in pattern.find_iter(&text) {
            //                     value = pattern.replace(mat.as_str(), value).to_string();
            //                 }
            //                 value
            //             };
            //
            //             return Ok(Some(ChangelogGroup {
            //                 group: parser.group.as_ref().cloned().map(regex_replace),
            //                 scope: parser.scope.as_ref().cloned().map(regex_replace),
            //                 default_scope: parser.default_scope.as_ref().cloned(),
            //             }));
            //         }
            //     } else {
            //         warn!(
            //             "{}",
            //             format!("Skip group parser for field {field} does not have a value")
            //         )
            //     }
            // }
        }

        Ok(None)
    }

    fn extract_links(commit: &ChangelogCommit, parsers: &[LinkParser]) -> Vec<Link> {
        let mut links = vec![];
        for parser in parsers {
            let regex = &parser.pattern;
            let replace = &parser.href;
            for mat in regex.find_iter(&commit.message) {
                let m = mat.as_str();
                let text = if let Some(text_replace) = &parser.text {
                    regex.replace(m, text_replace).to_string()
                } else {
                    m.to_string()
                };
                let href = regex.replace(m, replace);
                links.push(Link {
                    text,
                    href: href.to_string(),
                });
            }
        }

        links
    }

    pub fn new(commit: ChangelogCommit) -> Self {
        Self {
            commit,
            group: None,
            default_scope: None,
            scope: None,
            matched_group_parser: false,
            links: vec![],
        }
    }

    pub fn id(&self) -> &str {
        self.commit.id.as_str()
    }

    pub fn message(&self) -> &str {
        self.commit.message.as_str()
    }

    pub fn description(&self) -> &str {
        self.commit.description.as_str()
    }

    pub fn body(&self) -> &str {
        self.commit.body.as_str()
    }

    pub fn footers(&self) -> Option<&Vec<Footer>> {
        self.commit.footers.as_ref()
    }

    pub fn timestamp(&self) -> i64 {
        self.commit.timestamp
    }

    pub fn commit_type(&self) -> Option<&String> {
        self.commit.commit_type.as_ref()
    }

    pub fn scope(&self) -> Option<&String> {
        self.commit.scope.as_ref()
    }

    pub fn author(&self) -> &ScmSignature {
        &self.commit.author
    }

    pub fn committer(&self) -> &ScmSignature {
        &self.commit.committer
    }

    // pub fn id(&self) -> &str {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.id.as_str(),
    //         ChangelogCommit::Conventional(c) => c.commit.id.as_str(),
    //         ChangelogCommit::ReleaseNote(c) => c.commit.id.as_str(),
    //     }
    // }
    //
    // pub fn message(&self) -> &str {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.message.as_str(),
    //         ChangelogCommit::Conventional(c) => c.commit.message.as_str(),
    //         ChangelogCommit::ReleaseNote(c) => c.commit.message.as_str(),
    //     }
    // }
    //
    // pub fn description(&self) -> &str {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.description.as_str(),
    //         ChangelogCommit::Conventional(c) => c.commit.description.as_str(),
    //         ChangelogCommit::ReleaseNote(c) => c.commit.description.as_str(),
    //     }
    // }
    //
    // pub fn body(&self) -> &str {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.body.as_str(),
    //         ChangelogCommit::Conventional(c) => c.commit.body.as_str(),
    //         ChangelogCommit::ReleaseNote(c) => c.commit.body.as_str(),
    //     }
    // }
    //
    // pub fn footers(&self) -> Option<Vec<Footer>> {
    //     // TODO: parse footers for ScmCommit
    //     match &self.commit {
    //         ChangelogCommit::Standard(_) => None,
    //         ChangelogCommit::Conventional(c) => {
    //             Some(c.conv.footers().iter().map(Footer::from).collect())
    //         }
    //         ChangelogCommit::ReleaseNote(_) => None,
    //     }
    // }
    //
    // pub fn timestamp(&self) -> i64 {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.timestamp,
    //         ChangelogCommit::Conventional(c) => c.commit.timestamp,
    //         ChangelogCommit::ReleaseNote(c) => c.commit.timestamp,
    //     }
    // }
    //
    // pub fn commit_type(&self) -> Option<&str> {
    //     match &self.commit {
    //         ChangelogCommit::Standard(_) => None,
    //         ChangelogCommit::Conventional(c) => Some(c.conv.type_().as_str()),
    //         ChangelogCommit::ReleaseNote(_) => None,
    //     }
    // }
    //
    // pub fn scope(&self) -> Option<&str> {
    //     match &self.commit {
    //         ChangelogCommit::Standard(_) => None,
    //         ChangelogCommit::Conventional(c) => c.conv.scope().map(|s| s.as_str()),
    //         ChangelogCommit::ReleaseNote(_) => None,
    //     }
    // }
    //
    // pub fn author(&self) -> &ScmSignature {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.author.borrow(),
    //         ChangelogCommit::Conventional(c) => c.commit.author.borrow(),
    //         ChangelogCommit::ReleaseNote(c) => c.commit.author.borrow(),
    //     }
    // }
    //
    // pub fn committer(&self) -> &ScmSignature {
    //     match &self.commit {
    //         ChangelogCommit::Standard(c) => c.committer.borrow(),
    //         ChangelogCommit::Conventional(c) => c.commit.committer.borrow(),
    //         ChangelogCommit::ReleaseNote(c) => c.commit.committer.borrow(),
    //     }
    // }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Link {
    /// Text of the link.
    pub text: String,
    /// URL of the link
    pub href: String,
}

/// Commit signature that indicates authorship.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Signature {
    /// Name on the signature.
    pub name: Option<String>,
    /// Email on the signature.
    pub email: Option<String>,
    /// Time of the signature.
    pub timestamp: i64,
}

impl From<CommitSignature<'_>> for Signature {
    fn from(signature: CommitSignature) -> Self {
        Self {
            name: signature.name().map(String::from),
            email: signature.email().map(String::from),
            timestamp: signature.when().seconds(),
        }
    }
}

impl From<ScmSignature> for Signature {
    fn from(signature: ScmSignature) -> Self {
        Self {
            name: signature.name,
            email: signature.email,
            timestamp: signature.timestamp,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Footer {
    token: String,
    separator: String,
    value: String,
    breaking: bool,
}

impl<'a> Display for Footer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Self {
            token,
            separator,
            value,
            ..
        } = self;
        write!(f, "{token}{separator}{value}")
    }
}

impl<'a> From<&'a ConventionalFooter<'a>> for Footer {
    fn from(footer: &'a ConventionalFooter<'a>) -> Self {
        Self {
            token: footer.token().to_string(),
            separator: footer.separator().to_string(),
            value: footer.value().to_string(),
            breaking: footer.breaking(),
        }
    }
}

// impl Commit<'_> {
//     /// Processes the commit.
//     ///
//     /// * converts commit to a conventional commit
//     /// * sets the group for the commit
//     /// * extracts links and generates URLs
//     pub fn process(&self) -> CliResult<()> {
//         Ok(())
//     }
//
//     // into_conventional
//
//     // preprocess
//
//     // skip_commit
//
//     // parse
//
//     // parse_links
// }
//
// impl From<GitCommit<'_>> for Commit<'_> {
//     fn from(value: GitCommit) -> Self {
//         Self {
//             id: value.id().to_string(),
//             message: value.message().unwrap_or_default().to_string(),
//             commit: (),
//             group: None,
//             default_scope: None,
//             scope: None,
//             timestamp: 0,
//             links: vec![],
//             author: value.author().into(),
//             committer: value.committer().into(),
//             merge_commit: value.parents().count() > 1,
//         }
//     }
// }
//
// impl From<&ScmCommit> for Commit<'_> {
//     fn from(value: &ScmCommit) -> Self {
//         Self {
//             id: value.id.to_string(),
//             // TODO: way to avoid clones?
//             message: value.message.clone().unwrap_or_default(),
//             commit: (),
//             group: None,
//             default_scope: None,
//             author: value.author.clone().into(),
//             committer: value.committer.clone().into(),
//             // TODO: merge_commit
//             merge_commit: false,
//             timestamp: value.timestamp,
//             scope: None,
//             links: vec![],
//         }
//     }
// }
