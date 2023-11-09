use std::fs;

use regex::RegexBuilder;
use serde_derive::Serialize;
use serde_json::Value;

use crate::framework::{
    FrameworkDetectionItem, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::projects::msbuild::MsBuildProj;
use crate::projects::project_file::ProjectFile;
use crate::CifrsResult;

// Should we make detection a trait?
// pub trait Detection {
//     type Item;
//     // type Response;
//
//     // fn detect_framework(&self, Vec<Self::Item>) -> Option<Self::Item>;
//     // matches
//     // check
// }

// Return matched Framework
// which should have framework info
// as well as project

pub trait Dectector {

    fn detect<T>(&self) -> T;

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

pub(crate) fn detect_framework(
    frameworks: Vec<Box<dyn FrameworkSupport>>,
) -> Option<Box<dyn FrameworkSupport>> {
    for framework in frameworks {
        let m = matches(framework.get_info());
        // TODO: return MatchResult?
        if m.is_some() {
            return Some(framework);
        }
    }

    None
}

fn matches(framework: &FrameworkInfo) -> Option<MatchResult> {
    let mut results: Vec<Option<MatchResult>> = vec![];

    match &framework.detection.matching_strategy {
        FrameworkMatchingStrategy::All => {
            for detector in &framework.detection.detectors {
                results.push(check(framework, detector));
            }
        }
        FrameworkMatchingStrategy::Any => {
            let mut matched = None;
            for item in &framework.detection.detectors {
                let result = check(framework, item);
                if result.is_some() {
                    matched = result;
                    break;
                }
            }

            results.push(matched);
        }
    }

    if results.iter().all(|&r| r.is_some()) {
        return *results.first().unwrap();
    }

    None
}

// TODO: what should this return?
fn check(framework: &FrameworkInfo, item: &FrameworkDetectionItem) -> Option<MatchResult> {
    match item {
        FrameworkDetectionItem::Config { content } => {
            for config in &framework.configs {
                if let Ok(file_content) = fs::read_to_string(config) {
                    if let Some(content) = content {
                        let regex = RegexBuilder::new(content).multi_line(true).build();
                        match regex {
                            Ok(regex) => {
                                if regex.is_match(file_content.as_str()) {
                                    return Some(MatchResult { project: None });
                                }
                            }
                            Err(e) => {
                                // TODO: log
                            }
                        }
                    }
                    return Some(MatchResult { project: None });
                }
            }
            None
        }
        FrameworkDetectionItem::Dependency { name: dependency } => {
            for p in framework.backend.project_files() {
                for path in p.get_project_paths() {
                    if !path.exists() {
                        // TODO: log
                        continue;
                    }

                    if path.is_dir() {
                        // TODO: log
                        continue;
                    }

                    let file_content = fs::read_to_string(path);
                    match file_content {
                        Ok(c) => {
                            let found = has_dependency(p, c, dependency);
                            match found {
                                Ok(f) => {
                                    if f {
                                        return Some(MatchResult { project: Some(*p) });
                                    } else {
                                        // TODO: log -- dependency not found
                                    }
                                }
                                Err(_) => {
                                    // TODO: log -- error checking file for dependency
                                }
                            }
                        }
                        Err(e) => {
                            // TODO: log -- error reading file
                            continue;
                        }
                    }
                }
            }
            None
        }
        FrameworkDetectionItem::File { path, content } => {
            if let Ok(file_content) = fs::read_to_string(path) {
                if let Some(content) = content {
                    let regex = RegexBuilder::new(content).multi_line(true).build();
                    match regex {
                        Ok(regex) => {
                            if regex.is_match(file_content.as_str()) {
                                return Some(MatchResult { project: None });
                            }
                        }
                        Err(e) => {
                            // TODO: log
                            println!("error with regex {e}")
                        }
                    }
                }
                return Some(MatchResult { project: None });
            }
            None
        }
    }
}

fn has_dependency(
    project_type: &ProjectFile,
    content: String,
    dependency: &str,
) -> CifrsResult<bool> {
    let found = match project_type {
        ProjectFile::CargoToml => {
            let root: toml::Value = toml::from_str(content.as_str())?;
            root["dependencies"][dependency].is_str() || root["dev-dependencies"][dependency].is_str()
        }
        ProjectFile::MsBuild => {
            let build_proj: MsBuildProj = serde_xml_rs::from_str(content.as_str())?;
            build_proj.has_package_reference(dependency)
        }
        ProjectFile::GemFile => content.contains(&format!("gem '{dependency}'")),
        ProjectFile::GoMod => content.contains(&dependency.to_string()),
        ProjectFile::PackageJson => {
            let root: Value = serde_json::from_str(content.as_str())?;
            !root["dependencies"][dependency].is_null() || !root["devDependencies"][dependency].is_null()
        }
        ProjectFile::PipFile => {
            let root: toml::Value = toml::from_str(content.as_str())?;
            root["packages"][dependency].is_str() || root["dev-packages"][dependency].is_str()
        }
        ProjectFile::PyProject => {
            let root: toml::Value = toml::from_str(content.as_str())?;
            root["tool.poetry.dependencies"][dependency].is_str()
        }
        ProjectFile::RequirementsTxt => content.contains(&format!("{dependency}==")),
    };

    Ok(found)
}
