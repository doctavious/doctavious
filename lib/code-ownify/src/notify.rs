use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs, io, path};

use regex::Regex;
use scm::commit::ScmCommitRange;
use scm::drivers::git::GitScmRepository;
use scm::drivers::{Scm, ScmRepository};
use scm::errors::ScmError;
use thiserror::Error;
use tracing::{debug, info};
use walkdir::{DirEntry, WalkDir};

use crate::parse::pattern_to_regex;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CodeNotifyError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("SCM error: {0}")]
    ScmError(#[from] ScmError),

    #[error("Strip path error: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
}

pub type CodeNotifyResult<T> = Result<T, CodeNotifyError>;

// fn get_code_notify_files(cwd: &Path) -> impl Iterator<Item = DirEntry> {
//     WalkDir::new(cwd)
//         .into_iter()
//         .filter_entry(|e| {
//             if e.path().is_file() {
//                 return e.file_name().to_string_lossy() == "CODENOTIFY";
//             }
//
//             false
//         })
//         .filter_map(Result::ok)
// }

pub struct CodeNotify {
    // TODO: should this be an enum?
    /// The format of the output (text or markdown)
    pub format: String,

    /// The filename in which file subscribers are defined (Default: CODENOTIFY)
    pub file_name: String,

    /// The threshold for notifying subscribers (Default: 0)
    pub subscriber_threshold: i8,

    /// The commit range to use when computing the file diff
    pub commit_range: ScmCommitRange,

    /// The author of the file diff
    pub author: String,
}

// TODO: Provider support - start with github actions (env: GITHUB_ACTIONS)
impl CodeNotify {
    pub fn notify(&self) -> CodeNotifyResult<()> {
        let mut writer = io::stdout();
        self.notify_with_writer(&mut writer)?;
        Ok(())
    }

    fn notify_with_writer<W: Write>(&self, writer: &mut W) -> CodeNotifyResult<()> {
        let scm = Scm::get(&env::current_dir()?)?;
        let paths = scm.diff_paths(Some(&self.commit_range))?;
        self.inner_notify(writer, paths)?;

        Ok(())
    }

    fn inner_notify<W: Write>(&self, writer: &mut W, paths: Vec<PathBuf>) -> CodeNotifyResult<()> {
        let notifs = self.notifications(&paths, &self.file_name)?;
        self.write_notifications(writer, notifs)?;

        Ok(())
    }

    fn notifications(
        &self,
        paths: &Vec<PathBuf>,
        notify_filename: &str,
    ) -> CodeNotifyResult<HashMap<String, Vec<String>>> {
        let mut notifications: HashMap<String, Vec<String>> = HashMap::new();
        let root = Path::new(".");
        for p in paths {
            // We need to add root to paths as they dont contain it and we want to make sure we
            // check for codenotify files there.
            let full_path = root.join(p);
            let subs = self.subscribers(&full_path, notify_filename)?;
            for sub in subs {
                notifications
                    .entry(sub)
                    .or_insert(Vec::new())
                    .push(p.to_string_lossy().to_string());
            }
        }

        Ok(notifications)
    }

    fn write_notifications<W: Write>(
        &self,
        writer: &mut W,
        notifications: HashMap<String, Vec<String>>,
    ) -> CodeNotifyResult<()> {
        if self.subscriber_threshold > 0 && notifications.len() > self.subscriber_threshold as usize
        {
            info!("Not notifying subscribers as the number of notifying subscribers {} exceeds the threshold {}", notifications.len(), self.subscriber_threshold);
            return Ok(());
        }

        // TODO: set capacity? Improve this
        let mut subs = Vec::new();
        for sub in notifications.keys() {
            subs.push(sub)
        }

        subs.sort();

        match self.format.as_str() {
            "text" => {
                writeln!(
                    writer,
                    "{}...{}",
                    &self.commit_range.0,
                    &self
                        .commit_range
                        .1
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                )?;
                if subs.is_empty() {
                    write!(writer, "No notifications")?;
                } else {
                    for sub in subs {
                        if let Some(files) = notifications.get(sub) {
                            writeln!(writer, "{} -> {}", sub, files.join(", "))?;
                        }
                    }
                }
            }
            "markdown" => {
                write!(writer, "{}", self.markdown_comment_title(&self.file_name))?;
                write!(
                    writer,
                    "[CodeNotify](https://github.com/doctavious): Notifying subscribers in {} files for diff {}...{}.\n\n",
                    &self.file_name,
                    &self.commit_range.0,
                    &self
                        .commit_range
                        .1
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                )?;
                if subs.is_empty() {
                    write!(writer, "No notifications")?;
                } else {
                    writeln!(writer, "| Notify | File(s) |")?;
                    writeln!(writer, "|-|-|")?;
                    for sub in subs {
                        if let Some(files) = notifications.get(sub) {
                            writeln!(writer, "| {} | {} |", sub, files.join("<br>"))?;
                        }
                    }
                }
            }
            _ => println!("something else!"), // return error
        }

        Ok(())
    }

    fn subscribers(&self, path: &Path, notify_filename: &str) -> CodeNotifyResult<Vec<String>> {
        debug!("analyzing subscribers in {} files", notify_filename);
        let mut subscribers = Vec::new();

        let mut current_path = PathBuf::new();
        for component in path.components() {
            current_path.push(component);
            if current_path.is_dir() {
                let rule_path = current_path.join(notify_filename);
                if rule_path.is_file() {
                    for line in fs::read_to_string(&rule_path)?.lines() {
                        let trimmed_line = line.trim();
                        if trimmed_line.is_empty() || trimmed_line.starts_with("#") {
                            continue;
                        }
                        let fields: Vec<String> =
                            line.split_whitespace().map(str::to_string).collect();
                        if fields.len() == 1 {
                            info!(
                                "expected at least two fields for rule in {}: {}",
                                &rule_path.to_string_lossy(),
                                line
                            );
                            continue;
                        }

                        let relative = path.strip_prefix(&current_path)?;
                        let (rule_pattern, rest) =
                            fields.split_first().expect("Rule should have a pattern");
                        let re = pattern_to_regex(rule_pattern)?;
                        if re.is_match(&relative.to_string_lossy()) {
                            subscribers.extend(rest.to_vec());
                        }
                    }
                }
            }
        }

        Ok(subscribers)
    }

    fn markdown_comment_title(&self, file_name: &str) -> String {
        format!("<!-- codenotify:{} report -->\n", file_name)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use scm::commit::ScmCommitRange;
    use scm::drivers::git::GitScmRepository;
    use scm::drivers::ScmRepository;
    use tempfile::TempDir;
    use testing::guard::CwdGuard;

    use super::CodeNotify;

    #[test]
    fn basic() {
        let temp_dir = TempDir::new().unwrap();
        let guard = CwdGuard::new(&temp_dir);

        fs::write("CODENOTIFY", "**/*.md @markdown").unwrap();
        fs::write("file.md", "").unwrap();

        let scm = GitScmRepository::init(&temp_dir).expect("init git");
        scm.am("init").unwrap();
        let br = scm.get_commit_hash("HEAD").unwrap();

        fs::write("file.md", "foo").unwrap();
        scm.am("hr").unwrap();
        let hr = scm.get_commit_hash("HEAD").unwrap();

        let codenotify = CodeNotify {
            format: "text".to_string(),
            file_name: "CODENOTIFY".to_string(),
            subscriber_threshold: 0,
            // TODO: avoid these clones
            commit_range: ScmCommitRange(br.clone(), Some(hr.clone())),
            author: "".to_string(),
        };

        let mut writer = Vec::<u8>::new();
        codenotify.notify_with_writer(&mut writer).unwrap();

        assert_eq!(
            str::from_utf8(&writer).unwrap(),
            format!("{}...{}\n@markdown -> file.md\n", &br, &hr)
        );
    }

    #[test]
    fn test_write_notifications() {
        struct TestScenario {
            pub name: &'static str,
            pub opts: CodeNotify,
            pub notifs: HashMap<String, Vec<String>>,
            pub err: String,
            pub output: Vec<&'static str>, // pub output: &'static str,
        }

        let test_scenarios = vec![
            TestScenario {
                name: "empty markdown",
                opts: CodeNotify {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: "".to_string(),
                },
                notifs: Default::default(),
                err: "".to_string(),
                output: vec![
                    "<!-- codenotify:CODENOTIFY report -->",
                    "[CodeNotify](https://github.com/doctavious): Notifying subscribers in CODENOTIFY files for diff a...b.",
                    "",
                    "No notifications",
                ],
            },
            TestScenario {
                name: "empty text",
                opts: CodeNotify {
                    file_name: "CODENOTIFY".to_string(),
                    format: "text".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: "".to_string(),
                },
                notifs: Default::default(),
                err: "".to_string(),
                output: vec![
                    "a...b",
                    "No notifications",
                ],
            },
            TestScenario {
                name: "markdown",
                opts: CodeNotify {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: "".to_string(),
                },
                notifs: HashMap::from([
                    ("@go".to_string(), vec!["file.go".to_string(), "dir/file.go".to_string()]),
                    ("@js".to_string(), vec!["file.js".to_string(), "dir/file.js".to_string()]),
                ]),
                err: "".to_string(),
                output: vec![
                    "<!-- codenotify:CODENOTIFY report -->",
                    "[CodeNotify](https://github.com/doctavious): Notifying subscribers in CODENOTIFY files for diff a...b.",
                    "",
                    "| Notify | File(s) |",
                    "|-|-|",
                    "| @go | file.go<br>dir/file.go |",
                    "| @js | file.js<br>dir/file.js |",
                    ""
                ],
            }
        ];

        for t in test_scenarios {
            println!("{}", t.name);
            let mut writer = Vec::<u8>::new();
            let r = t.opts.write_notifications(&mut writer, t.notifs);
            if r.is_err() && t.err == "" {
                // TODO: fail - "expected nil error; got %s"
            }

            if r.is_ok() && t.err != "" {
                // TODO: fail - "expected error %q; got nil", test.err
            }

            let actual_output = String::from_utf8(writer).unwrap();
            assert_eq!(actual_output, t.output.join("\n"));
        }
    }
}
