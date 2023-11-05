use std::fs;
use std::path::{Path, PathBuf};

use directories::{UserDirs};
use glob::PatternError;
use regex::RegexBuilder;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::framework::{FrameworkDetectionItem, FrameworkInfo, FrameworkMatchingStrategy};
use crate::framework_detection::MatchResult;
use crate::frameworks::FRAMEWORKS_STR;
use crate::projects::csproj::CSProj;
use crate::projects::project_file::ProjectFile;

mod backends;
mod framework;
mod framework_detection;
mod frameworks;
mod js_module;
mod language;
mod package_manager;
mod projects;
mod strategy;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CifrsError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("glob parsing pattern error: {0}")]
    GlobPattern(#[from] PatternError),

    #[error("Invalid config: {0}")]
    InvalidConfig(String),

    /// Error that may occur while I/O operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Framework config not found. Config must have an extension of json, yaml, toml, or js|cjs|mjs")]
    MissingFrameworkConfig(),

    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("xml serialize/deserialize error: {0}")]
    SerdeXml(#[from] serde_xml_rs::Error),

    #[error("yaml serialize/deserialize error: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    /// Errors that may occur when deserializing types from TOML format.
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Errors that may occur when serializing types from TOML format.
    #[error("toml serialization error: `{0}`")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Unknown framework extension: {0}")]
    UnknownFrameworkExtension(String),
}

pub type CifrsResult<T> = Result<T, CifrsError>;

pub struct Cifrs;

#[derive(Debug, Deserialize, Serialize)]
struct Frameworks {
    pub frameworks: Vec<FrameworkInfo>,
}

impl Cifrs {
    // TODO: when looking for frameworks do we need to traverse more than one directory?
    // That might be only true if project is a monorepo? See how projects with included docs are built
    // Initial thoughts are
    // get all frameworks
    // collect root and top level directories
    // for each directory check the framework. Initially thought about organizing frameworks by
    // project file and config so didnt have to check every framework. this might still be a good
    // idea. at build or runtime collect by project and check each.
    // if there is 1 great success. otherwise for each check by config
    // if there is 1 great success. otherwise can fail or attempt to build each one
    // need to tie package_managers / to frameworks. Language?
    // the problem is that just because you found a project doesnt mean its related to docs and
    // another project is used for the framework

    /// Determine Frameworks
    /// returns vec of frameworks
    pub fn check_frameworks<P: AsRef<Path>>(path: P) -> CifrsResult<FrameworkInfo> {
        let frameworks: Frameworks = serde_yaml::from_str(FRAMEWORKS_STR)?;

        // TODO: should we decide if monorepo / workspace?
        // vercel build has a max depth of 3. should we try and determine max depth
        // based on if its a monorepo/workspace or individual project?
        let dirs = Cifrs::directories_to_check(path)?;
        for framework in frameworks.frameworks {
            for dir in &dirs {
                let m = Cifrs::matches(&framework, dir);
                // TODO: return MatchResult?
                if m.is_some() {
                    return Ok(framework);
                }
            }
        }

        Err(CifrsError::MissingFrameworkConfig())
    }

    pub fn build<P: AsRef<Path>>(path: P, install: bool) -> CifrsResult<()> {
        let dirs = Cifrs::directories_to_check(path);
        Ok(())
    }

    pub fn directories_to_check<P: AsRef<Path>>(path: P) -> CifrsResult<Vec<PathBuf>> {
        let mut dirs = vec![path.as_ref().to_path_buf()];
        // for entry in fs::read_dir(path)?.flatten() {
        //     if entry.path().is_dir() {
        //         dirs.push(entry.path());
        //     }
        // }

        Ok(dirs)
    }

    fn matches(framework: &FrameworkInfo, path: &Path) -> Option<MatchResult> {
        let mut results: Vec<Option<MatchResult>> = vec![];

        match &framework.detection.matching_strategy {
            FrameworkMatchingStrategy::All => {
                for detector in &framework.detection.detectors {
                    results.push(Cifrs::check(&framework, detector, path));
                }
            }
            FrameworkMatchingStrategy::Any => {
                let mut matched = None;
                for item in &framework.detection.detectors {
                    let result = Cifrs::check(&framework, item, path);
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
    fn check(
        framework: &FrameworkInfo,
        item: &FrameworkDetectionItem,
        root: &Path
    ) -> Option<MatchResult> {
        match item {
            FrameworkDetectionItem::Config { content } => {
                for config in &framework.configs {
                    if let Ok(file_content) = fs::read_to_string(root.join(config)) {
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
                    for project_path in p.get_project_paths() {
                        let path = root.join(project_path);
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
                                let found = Cifrs::has_dependency(p, c, dependency);
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
                if let Ok(file_content) = fs::read_to_string(root.join(path)) {
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
                // TODO: do we want to check dev-packages
                root.get("dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::CSProj => {
                let build_proj: CSProj = serde_xml_rs::from_str(content.as_str())?;
                build_proj.has_package_reference(dependency)
            }
            ProjectFile::GemFile => content.contains(&format!("gem '{}'", dependency)),
            ProjectFile::GoMod => content.contains(dependency),
            ProjectFile::PackageJson => {
                let root: Value = serde_json::from_str(content.as_str())?;
                // TODO: do we want to check devDependencies
                root.get("dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::PipFile => {
                let root: toml::Value = toml::from_str(content.as_str())?;
                // TODO: do we want to check dev-packages
                root.get("packages")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::PyProject => {
                let root: toml::Value = toml::from_str(content.as_str())?;
                // might be to do these individual lookup
                root.get("tool.poetry.dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::RequirementsTxt => content.contains(&format!("{}==", dependency)),
        };

        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use directories::BaseDirs;
    use crate::Cifrs;

    #[test]
    fn check_frameworks() {
        let base_dir = BaseDirs::new().unwrap();
        let home_dir = base_dir.home_dir();
        let framework = Cifrs::check_frameworks(&home_dir.join("workspace/seancarroll.github.io")).unwrap();
        println!("{:?}", framework)
    }

    // #[test]
    // fn build() {
    //
    // }
}