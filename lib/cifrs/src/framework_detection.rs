use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use regex::RegexBuilder;
use serde_derive::Serialize;
use tracing::{debug, error};

use crate::frameworks::{FrameworkDetectionItem, FrameworkInfo, FrameworkMatchingStrategy};
use crate::projects::project_file::ProjectFile;

// Return matched Framework
// which should have framework info
// as well as project

pub trait Detectable {
    // this could be something like a Match / MatchResult
    // fn detect<T>(&self) -> Option<T>;

    // could just expose FrameworkDetector

    fn get_matching_strategy(&self) -> &FrameworkMatchingStrategy;

    fn get_detectors(&self) -> &Vec<FrameworkDetectionItem>;

    fn get_project_files(&self) -> Cow<Vec<ProjectFile>>;

    fn get_configuration_files(&self) -> &Vec<PathBuf>;
}

pub(crate) struct MatchedFramework<'a> {
    pub framework_info: &'a FrameworkInfo,
    pub project: Option<ProjectFile>,
}

#[derive(Clone, Copy, Serialize)]
pub(crate) struct MatchResult {
    // dependency -- could also do a dependency/version struct tuple and have an array of them
    // detected_version: String
    pub project: Option<ProjectFile>,
}

// TODO: Probably wont use this and will use one in lib.rs
pub(crate) fn detect_framework<P: AsRef<Path>, T: Detectable>(
    cwd: P,
    frameworks: Vec<T>,
) -> Option<T> {
    for framework in frameworks {
        let match_result = detect(&cwd, &framework);
        if match_result.is_some() {
            return Some(framework);
        }
    }

    None
}

pub(crate) fn detect<P: AsRef<Path>, T: Detectable>(cwd: P, framework: &T) -> Option<MatchResult> {
    let mut results: Vec<Option<MatchResult>> = vec![];

    match framework.get_matching_strategy() {
        FrameworkMatchingStrategy::All => {
            for detector in framework.get_detectors() {
                results.push(check(&cwd, framework, detector));
            }
        }
        FrameworkMatchingStrategy::Any => {
            let mut matched = None;
            for item in framework.get_detectors() {
                let result = check(&cwd, framework, item);
                if result.is_some() {
                    matched = result;
                    break;
                }
            }

            results.push(matched);
        }
    }

    if results.iter().all(|&r| r.is_some()) {
        return *results.first()?;
    }

    None
}

// TODO: should this return a result?
fn check<'a, P: AsRef<Path>, T: Detectable>(
    cwd: P,
    framework: &'a T,
    item: &'a FrameworkDetectionItem,
) -> Option<MatchResult> {
    match item {
        FrameworkDetectionItem::Config { content } => {
            for config in framework.get_configuration_files() {
                if check_file(
                    cwd.as_ref().join(config),
                    content.as_ref().map(|x| x.as_str()),
                )
                .is_some()
                {
                    return Some(MatchResult { project: None });
                }
            }
            None
        }
        FrameworkDetectionItem::Dependency { name: dependency } => {
            for p in framework.get_project_files().iter() {
                match p.has_dependency(&cwd, dependency) {
                    Ok(found) => {
                        if found {
                            return Some(MatchResult { project: Some(*p) });
                        } else {
                            debug!("Failed to find dependency {dependency} in project {:?}", p);
                        }
                    }
                    Err(e) => {
                        error!("Error getting dependency from project file {:?}: {}", p, e);
                    }
                }
            }
            None
        }
        FrameworkDetectionItem::File { path, content } => {
            if check_file(
                cwd.as_ref().join(path),
                content.as_ref().map(|x| x.as_str()),
            )
            .is_some()
            {
                return Some(MatchResult { project: None });
            }

            None
        }
    }
}

fn check_file<'a, P: AsRef<Path>>(path: P, pattern: Option<&str>) -> Option<MatchResult> {
    if let Ok(file_content) = fs::read_to_string(path) {
        if let Some(pattern) = pattern {
            let reg = RegexBuilder::new(pattern)
                .multi_line(true)
                .build()
                .expect("Pattern should be valid");
            if reg.is_match(&file_content) {
                return Some(MatchResult { project: None });
            }
        } else {
            return Some(MatchResult { project: None });
        }
    }

    None
}
