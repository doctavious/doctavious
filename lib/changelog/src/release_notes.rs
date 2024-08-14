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
    pub breaking_change: bool,
}

pub struct ReleaseNotes {
    pub breaking_change_category: String,
    // TODO: could we use commit / group parser to determine feature / minor / patch?
    // what does knowing feature really provide? Way to determine how to bump? if so maybe just make
    // it part of bump
}

impl ReleaseNotes {
    pub fn parse_commit(&self, commit: &ScmCommit) -> Vec<ReleaseNote> {
        let mut release_notes = vec![];
        for line in commit.message.lines() {
            if RE.is_match(line) {
                let captures = RE.captures(line).unwrap();
                let category = captures.name("category").map(|c| c.as_str().to_string());
                let description = captures
                    .name("description")
                    .map_or(String::new(), |c| c.as_str().to_string());

                let breaking_change = category
                    .as_ref()
                    .is_some_and(|c| *c == self.breaking_change_category);

                if description.to_lowercase().trim() == "none" {
                    release_notes.push(ReleaseNote {
                        category,
                        description,
                        commit: commit.clone(),
                        breaking_change,
                    });
                }
            }
        }

        release_notes
    }
}
