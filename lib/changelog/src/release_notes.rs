use lazy_static::lazy_static;
use regex::Regex;
use scm::commit::ScmCommit;
use serde_derive::{Deserialize, Serialize};

lazy_static! {
    static ref RE: Regex =
        Regex::new("^Release note(?<category> .+)?:(?<description> .+)").unwrap();
}

// Associated configuration should allow for category that defines breaking change with the default
// being something like `backward-incompatible change` or `breaking`
#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseNote {
    // TODO: change name to scope
    pub category: Option<String>,
    pub description: String,
    pub commit: ScmCommit,
    // TODO: breaking change - backward-incompatible change category
}

impl ReleaseNote {
    // TODO: support for determining breaking change
    pub fn parse_commit(commit: &ScmCommit) -> Vec<Self> {
        let mut release_notes = vec![];
        for line in commit.message.lines() {
            if RE.is_match(line) {
                let captures = RE.captures(line).unwrap();
                let category = captures.name("category").map(|c| c.as_str().to_string());
                let description = captures
                    .name("description")
                    .map_or(String::new(), |c| c.as_str().to_string());

                if description.to_lowercase().trim() == "none" {
                    release_notes.push(ReleaseNote {
                        category,
                        description,
                        commit: commit.clone(),
                    });
                }
            }
        }

        release_notes
    }
}
