use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io, path};

use scm::commit::ScmCommitRange;
use scm::drivers::{Scm, ScmRepository};
use scm::errors::ScmError;
use thiserror::Error;
use tracing::{debug, info};

use crate::parser;
use crate::parser::pattern_to_regex;

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
    StripPrefixError(#[from] path::StripPrefixError),

    #[error("Unsupported format {0}")]
    UnsupportedFormat(String),
}

pub type CodeNotifyResult<T> = Result<T, CodeNotifyError>;

pub struct CodeNotify {
    pub cwd: PathBuf,

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
    pub author: Option<String>,
}

// TODO: Provider support - start with github actions (env: GITHUB_ACTIONS)
impl CodeNotify {
    pub fn notify(&self) -> CodeNotifyResult<()> {
        let mut writer = io::stdout();
        self.notify_with_writer(&mut writer)?;
        Ok(())
    }

    fn notify_with_writer<W: Write>(&self, writer: &mut W) -> CodeNotifyResult<()> {
        let scm = Scm::get(&self.cwd)?;
        let paths = scm.diff_paths(Some(&self.commit_range))?;
        self.inner_notify(writer, paths)?;

        Ok(())
    }

    fn inner_notify<W: Write>(&self, writer: &mut W, paths: Vec<PathBuf>) -> CodeNotifyResult<()> {
        let notifs = self.notifications(&paths)?;
        self.write_notifications(writer, notifs)?;

        Ok(())
    }

    fn notifications(
        &self,
        paths: &Vec<PathBuf>,
    ) -> CodeNotifyResult<HashMap<String, Vec<String>>> {
        let mut notifications: HashMap<String, Vec<String>> = HashMap::new();
        let root = &self.cwd;
        for p in paths {
            // We need to add root to paths as they dont contain it and we want to make sure we
            // check for codenotify files there.
            let full_path = root.join(p);
            let subs = self.subscribers(&full_path)?;
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
            writeln!(
                writer,
                "Not notifying subscribers as the number of notifying subscribers {} exceeds the threshold {}",
                notifications.len(),
                self.subscriber_threshold
            )?;
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
            _ => return Err(CodeNotifyError::UnsupportedFormat(self.format.to_owned())),
        }

        Ok(())
    }

    fn subscribers(&self, path: &Path) -> CodeNotifyResult<Vec<String>> {
        debug!("analyzing subscribers in {} files", &self.file_name);
        let mut subscribers = Vec::new();

        let mut current_path = PathBuf::new();
        for component in path.components() {
            current_path.push(component);
            if current_path.is_dir() {
                let rule_path = current_path.join(&self.file_name);
                if rule_path.is_file() {
                    let relative = path.strip_prefix(&current_path)?;
                    for line in fs::read_to_string(&rule_path)?.lines() {
                        if let Some((rule_pattern, pattern_subscribers)) = parser::parse_line(line)
                        {
                            if pattern_subscribers.is_empty() {
                                info!(
                                    "expected subscribers for rule in {}: {}",
                                    &rule_path.to_string_lossy(),
                                    line
                                );
                                continue;
                            }

                            let re = pattern_to_regex(&rule_pattern)?;
                            if re.is_match(&relative.to_string_lossy()) {
                                subscribers.extend(pattern_subscribers);
                            }
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
    use std::path::PathBuf;

    use scm::commit::ScmCommitRange;
    use scm::drivers::git::GitScmRepository;
    use testing::guard::TempDirGuard;

    use super::CodeNotify;

    #[test]
    fn basic() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        fs::write(temp_dir.join("CODENOTIFY"), "**/*.md @markdown").unwrap();
        fs::write(temp_dir.join("file.md"), "").unwrap();

        let scm = GitScmRepository::init(&temp_dir).expect("init git");
        scm.am("init").unwrap();
        let br = scm.get_commit_hash("HEAD").unwrap();

        fs::write(temp_dir.join("file.md"), "foo").unwrap();
        scm.am("hr").unwrap();
        let hr = scm.get_commit_hash("HEAD").unwrap();

        let codenotify = CodeNotify {
            cwd: temp_dir,
            format: "text".to_string(),
            file_name: "CODENOTIFY".to_string(),
            subscriber_threshold: 0,
            // TODO: avoid these clones
            commit_range: ScmCommitRange(br.clone(), Some(hr.clone())),
            author: None,
        };

        let mut writer = Vec::<u8>::new();
        codenotify.notify_with_writer(&mut writer).unwrap();

        assert_eq!(
            str::from_utf8(&writer).unwrap(),
            format!("{}...{}\n@markdown -> file.md\n", &br, &hr)
        );
    }

    struct Opts {
        pub format: String,
        pub file_name: String,
        pub subscriber_threshold: i8,
        pub commit_range: ScmCommitRange,
        pub author: Option<String>,
    }

    impl Opts {
        fn to_codenotify(self, cwd: PathBuf) -> CodeNotify {
            CodeNotify {
                cwd,
                format: self.format,
                file_name: self.file_name,
                subscriber_threshold: self.subscriber_threshold,
                commit_range: self.commit_range,
                author: self.author,
            }
        }
    }

    #[test]
    fn test_write_notifications() {
        struct TestScenario {
            pub name: &'static str,
            pub opts: Opts,
            pub notifs: HashMap<String, Vec<String>>,
            pub err: Option<String>,
            pub output: Vec<&'static str>, // pub output: &'static str,
        }

        let test_scenarios = vec![
            TestScenario {
                name: "empty markdown",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                notifs: Default::default(),
                err: None,
                output: vec![
                    "<!-- codenotify:CODENOTIFY report -->",
                    "[CodeNotify](https://github.com/doctavious): Notifying subscribers in CODENOTIFY files for diff a...b.",
                    "",
                    "No notifications",
                ],
            },
            TestScenario {
                name: "empty text",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "text".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                notifs: Default::default(),
                err: None,
                output: vec!["a...b", "No notifications"],
            },
            TestScenario {
                name: "markdown",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                notifs: HashMap::from([
                    (
                        "@go".to_string(),
                        vec!["file.go".to_string(), "dir/file.go".to_string()],
                    ),
                    (
                        "@js".to_string(),
                        vec!["file.js".to_string(), "dir/file.js".to_string()],
                    ),
                ]),
                err: None,
                output: vec![
                    "<!-- codenotify:CODENOTIFY report -->",
                    "[CodeNotify](https://github.com/doctavious): Notifying subscribers in CODENOTIFY files for diff a...b.",
                    "",
                    "| Notify | File(s) |",
                    "|-|-|",
                    "| @go | file.go<br>dir/file.go |",
                    "| @js | file.js<br>dir/file.js |",
                    "",
                ],
            },
            TestScenario {
                name: "text",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "text".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                notifs: HashMap::from([
                    (
                        "@go".to_string(),
                        vec!["file.go".to_string(), "dir/file.go".to_string()],
                    ),
                    (
                        "@js".to_string(),
                        vec!["file.js".to_string(), "dir/file.js".to_string()],
                    ),
                ]),
                err: None,
                output: vec![
                    "a...b",
                    "@go -> file.go, dir/file.go",
                    "@js -> file.js, dir/file.js",
                    "",
                ],
            },
            TestScenario {
                name: "unsupported format",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "pdf".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                notifs: HashMap::from([(
                    "@go".to_string(),
                    vec!["file.go".to_string(), "dir/file.go".to_string()],
                )]),
                err: Some("unsupported format: pdf".to_string()),
                output: vec![],
            },
            TestScenario {
                name: "exceed subscriber threshold",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "text".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 1,
                    author: None,
                },
                notifs: HashMap::from([
                    (
                        "@go".to_string(),
                        vec!["file.go".to_string(), "dir/file.go".to_string()],
                    ),
                    (
                        "@js".to_string(),
                        vec!["file.js".to_string(), "dir/file.js".to_string()],
                    ),
                ]),
                err: None,
                output: vec![
                    "Not notifying subscribers as the number of notifying subscribers 2 exceeds the threshold 1\n",
                ],
            },
        ];

        for t in test_scenarios {
            let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();
            let code_notify = t.opts.to_codenotify(temp_dir);

            let mut writer = Vec::<u8>::new();
            let r = code_notify.write_notifications(&mut writer, t.notifs);
            if r.is_err() && t.err.is_none() {
                panic!(
                    "expected ok result error; got {}",
                    r.err().unwrap().to_string()
                );
            }

            if r.is_ok() && t.err.is_some() {
                panic!("expected error {} but result was ok", t.err.unwrap());
            }

            let actual_output = String::from_utf8(writer).unwrap();
            assert_eq!(actual_output, t.output.join("\n"));
        }
    }

    #[test]
    fn test_notifications() {
        struct TestScenario {
            name: &'static str,
            opts: Opts,
            paths: HashMap<&'static str, &'static str>,
            notifications: HashMap<String, Vec<String>>,
        }

        let test_scenarios = vec![
            TestScenario {
                name: "no notifications",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "nomatch.md @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::default(),
            },
            TestScenario {
                name: "file.md",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "file.md @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "no leading slash",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "/file.md @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::default(),
            },
            TestScenario {
                name: "whitespace",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "\n\nfile.md @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "comments",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "#comment\nfile.md @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "*",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "* @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["CODENOTIFY".to_string(), "file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "dir/*",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "dir/* @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["dir/file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "**",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "** @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec![
                        "CODENOTIFY".to_string(),
                        "file.md".to_string(),
                        "dir/file.md".to_string(),
                        "dir/dir/file.md".to_string(),
                    ],
                )]),
            },
            // same as **
            TestScenario {
                name: "**/*",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "**/* @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec![
                        "CODENOTIFY".to_string(),
                        "file.md".to_string(),
                        "dir/file.md".to_string(),
                        "dir/dir/file.md".to_string(),
                    ],
                )]),
            },
            TestScenario {
                name: "**/file.md",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "**/file.md @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec![
                        "file.md".to_string(),
                        "dir/file.md".to_string(),
                        "dir/dir/file.md".to_string(),
                    ],
                )]),
            },
            TestScenario {
                name: "dir/**",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "dir/** @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["dir/file.md".to_string(), "dir/dir/file.md".to_string()],
                )]),
            },
            // same as "dir/**"
            TestScenario {
                name: "dir/",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "dir/** @notify\n"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["dir/file.md".to_string(), "dir/dir/file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "dir/**/file.md",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "dir/**/file.md @notify\n"),
                    ("file.md", ""),
                    ("dirfile.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["dir/file.md".to_string(), "dir/dir/file.md".to_string()],
                )]),
            },
            TestScenario {
                name: "multiple subscribers",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([("CODENOTIFY", "* @alice @bob\n"), ("file.md", "")]),
                notifications: HashMap::from([
                    (
                        "@alice".to_string(),
                        vec!["CODENOTIFY".to_string(), "file.md".to_string()],
                    ),
                    (
                        "@bob".to_string(),
                        vec!["CODENOTIFY".to_string(), "file.md".to_string()],
                    ),
                ]),
            },
            TestScenario {
                name: "..",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([("CODENOTIFY", "../* @alice @bob\n"), ("file.md", "")]),
                notifications: HashMap::default(),
            },
            TestScenario {
                name: "multiple CODENOTIFY",
                opts: Opts {
                    file_name: "CODENOTIFY".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    (
                        "CODENOTIFY",
                        "* @rootany\n*.go @rootgo\n*.js @rootjs\n**/* @all\n**/*.go @allgo\n**/*.js @alljs\n",
                    ),
                    ("file.md", ""),
                    ("file.js", ""),
                    ("file.go", ""),
                    (
                        "dir/CODENOTIFY",
                        "* @dir/any\n*.go @dir/go\n*.js @dir/js\n**/* @dir/all\n**/*.go @dir/allgo\n**/*.js @dir/alljs\n",
                    ),
                    ("dir/file.md", ""),
                    ("dir/file.js", ""),
                    ("dir/file.go", ""),
                    (
                        "dir/dir/CODENOTIFY",
                        "* @dir/dir/any\n*.go @dir/dir/go\n*.js @dir/dir/js\n**/* @dir/dir/all\n**/*.go @dir/dir/allgo\n**/*.js @dir/dir/alljs\n",
                    ),
                    ("dir/dir/file.md", ""),
                    ("dir/dir/file.js", ""),
                    ("dir/dir/file.go", ""),
                ]),
                notifications: HashMap::from([
                    (
                        "@all".to_string(),
                        vec![
                            "CODENOTIFY".to_string(),
                            "file.md".to_string(),
                            "file.js".to_string(),
                            "file.go".to_string(),
                            "dir/CODENOTIFY".to_string(),
                            "dir/file.md".to_string(),
                            "dir/file.js".to_string(),
                            "dir/file.go".to_string(),
                            "dir/dir/CODENOTIFY".to_string(),
                            "dir/dir/file.md".to_string(),
                            "dir/dir/file.js".to_string(),
                            "dir/dir/file.go".to_string(),
                        ],
                    ),
                    (
                        "@allgo".to_string(),
                        vec![
                            "file.go".to_string(),
                            "dir/file.go".to_string(),
                            "dir/dir/file.go".to_string(),
                        ],
                    ),
                    (
                        "@alljs".to_string(),
                        vec![
                            "file.js".to_string(),
                            "dir/file.js".to_string(),
                            "dir/dir/file.js".to_string(),
                        ],
                    ),
                    (
                        "@rootany".to_string(),
                        vec![
                            "CODENOTIFY".to_string(),
                            "file.md".to_string(),
                            "file.js".to_string(),
                            "file.go".to_string(),
                        ],
                    ),
                    ("@rootgo".to_string(), vec!["file.go".to_string()]),
                    ("@rootjs".to_string(), vec!["file.js".to_string()]),
                    (
                        "@dir/all".to_string(),
                        vec![
                            "dir/CODENOTIFY".to_string(),
                            "dir/file.md".to_string(),
                            "dir/file.js".to_string(),
                            "dir/file.go".to_string(),
                            "dir/dir/CODENOTIFY".to_string(),
                            "dir/dir/file.md".to_string(),
                            "dir/dir/file.js".to_string(),
                            "dir/dir/file.go".to_string(),
                        ],
                    ),
                    (
                        "@dir/allgo".to_string(),
                        vec!["dir/file.go".to_string(), "dir/dir/file.go".to_string()],
                    ),
                    (
                        "@dir/alljs".to_string(),
                        vec!["dir/file.js".to_string(), "dir/dir/file.js".to_string()],
                    ),
                    (
                        "@dir/any".to_string(),
                        vec![
                            "dir/CODENOTIFY".to_string(),
                            "dir/file.md".to_string(),
                            "dir/file.js".to_string(),
                            "dir/file.go".to_string(),
                        ],
                    ),
                    ("@dir/go".to_string(), vec!["dir/file.go".to_string()]),
                    ("@dir/js".to_string(), vec!["dir/file.js".to_string()]),
                    (
                        "@dir/dir/all".to_string(),
                        vec![
                            "dir/dir/CODENOTIFY".to_string(),
                            "dir/dir/file.md".to_string(),
                            "dir/dir/file.js".to_string(),
                            "dir/dir/file.go".to_string(),
                        ],
                    ),
                    (
                        "@dir/dir/allgo".to_string(),
                        vec!["dir/dir/file.go".to_string()],
                    ),
                    (
                        "@dir/dir/alljs".to_string(),
                        vec!["dir/dir/file.js".to_string()],
                    ),
                    (
                        "@dir/dir/any".to_string(),
                        vec![
                            "dir/dir/CODENOTIFY".to_string(),
                            "dir/dir/file.md".to_string(),
                            "dir/dir/file.js".to_string(),
                            "dir/dir/file.go".to_string(),
                        ],
                    ),
                    (
                        "@dir/dir/go".to_string(),
                        vec!["dir/dir/file.go".to_string()],
                    ),
                    (
                        "@dir/dir/js".to_string(),
                        vec!["dir/dir/file.js".to_string()],
                    ),
                ]),
            },
            TestScenario {
                name: "no notifications for OWNERS",
                opts: Opts {
                    file_name: "OWNERS".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "file.md @notify"),
                    ("OWNERS", "nomatch.md @notify"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::default(),
            },
            TestScenario {
                name: "file.md in OWNERS",
                opts: Opts {
                    file_name: "OWNERS".to_string(),
                    format: "markdown".to_string(),
                    commit_range: ScmCommitRange("a".to_string(), Some("b".to_string())),
                    subscriber_threshold: 0,
                    author: None,
                },
                paths: HashMap::from([
                    ("CODENOTIFY", "nomatch.md @notify"),
                    ("OWNERS", "file.md @notify"),
                    ("file.md", ""),
                    ("dir/file.md", ""),
                    ("dir/dir/file.md", ""),
                ]),
                notifications: HashMap::from([(
                    "@notify".to_string(),
                    vec!["file.md".to_string()],
                )]),
            },
        ];

        for mut t in test_scenarios {
            println!("Testing {}", t.name);
            let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

            let mut paths = vec![];
            for (p, contents) in t.paths.into_iter() {
                let path = temp_dir.join(p);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).unwrap();
                }
                fs::write(path, contents).unwrap();
                paths.push(PathBuf::from(p));
            }

            let code_notify = t.opts.to_codenotify(temp_dir);
            let mut notifs = code_notify.notifications(&paths).unwrap();

            for (subscriber, actual_files) in notifs.iter_mut() {
                let expected_files = t.notifications.get_mut(subscriber).unwrap();
                actual_files.sort();
                expected_files.sort();
                assert_eq!(expected_files, actual_files);
            }

            for (subscriber, expected_files) in t.notifications.iter_mut() {
                if notifs.contains_key(subscriber) {
                    continue;
                }

                let actual_files = notifs.get_mut(subscriber).unwrap();
                actual_files.sort();
                expected_files.sort();
                assert_eq!(expected_files, actual_files);
            }
        }
    }
}
