use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use glob::PatternError;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::framework::FrameworkInfo;
use crate::framework_detection::Detectable;
use crate::frameworks::FRAMEWORKS_STR;
use crate::package_manager::PackageManagerInfo;
use crate::workspaces::{Workspace, WORKSPACES_STR};

mod backends;
mod framework;
mod framework_detection;
mod frameworks;
mod js_module;
mod language;
mod package_manager;
mod projects;
mod workspaces;

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

    #[error("Unknown project file: {0}")]
    UnknownProjectFilePath(String),

    #[error("Unknown workspace implementation: {0}")]
    UnknownWorkspaceImplementation(String),
}

pub type CifrsResult<T> = Result<T, CifrsError>;

pub struct Cifrs;

#[derive(Debug, Deserialize, Serialize)]
struct SupportedFrameworks {
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
    pub fn detect_frameworks<P: AsRef<Path>>(path: P) -> CifrsResult<FrameworkInfo> {
        if !path.as_ref().is_dir() {
            // TODO: return error
        }

        let workspace = Cifrs::detect_workspace(&path)?;
        let project_paths = Cifrs::get_workspace_package_paths(&path, workspace)?;

        for dir in &project_paths {
            for framework in Cifrs::get_frameworks()?.frameworks {
                let m = framework_detection::detect(&dir, &framework);
                // TODO: return MatchResult?
                if m.is_some() {
                    return Ok(framework);
                }
            }
        }

        Err(CifrsError::MissingFrameworkConfig())
    }

    pub fn get_frameworks() -> CifrsResult<SupportedFrameworks> {
        Ok(serde_yaml::from_str(FRAMEWORKS_STR)?)
    }

    pub fn build<P: AsRef<Path>>(path: &P, install: bool) -> CifrsResult<()> {
        let framework = Cifrs::detect_frameworks(path)?;

        if install {
            let package_manager_info = Cifrs::detect_package_manager(&path, &framework);
            // TODO(Sean): if we can't get package_manager should this fail?
            if let Some(package_manager_info) = package_manager_info {
                let install_status = Cifrs::do_install(&package_manager_info)?;
                if !install_status.success() {
                    let install_status_code = install_status
                        .code()
                        .map_or("unknown".to_string(), |s| s.to_string());
                    println!("install failed with code: {:?}", install_status_code);
                    // TODO: probably should just use anyhow
                    return Err(CifrsError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("install failed with code: {:?}", install_status_code),
                    )));
                }
            }
        }

        let build_status = Cifrs::do_build(&framework)?;
        if !build_status.success() {
            let build_status_code = build_status
                .code()
                .map_or("unknown".to_string(), |s| s.to_string());
            println!("build failed with code: {:?}", build_status_code);
            // TODO: probably should just use anyhow
            return Err(CifrsError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("build failed with code: {:?}", build_status_code),
            )));
        }

        Ok(())
    }

    fn detect_package_manager<P: AsRef<Path>>(
        cwd: P,
        framework: &FrameworkInfo,
    ) -> Option<PackageManagerInfo> {
        // for workspace in &framework.backend {
        //     let m = framework_detection::detect(&cwd, &workspace);
        //     if m.is_some() {
        //         return Ok(Some(workspace));
        //     }
        // }

        None
    }

    // fn detect_package_manager<'a>(framework_info: &FrameworkInfo) -> Option<PackageManagerInfo> {
    //     let mut results: Vec<bool> = vec![];
    //     for package_manager in framework_info.language.get_package_managers() {
    //         let package_manager_info = package_manager.info();
    //         match package_manager_info.detection.matching_strategy {
    //             FrameworkMatchingStrategy::All => {
    //                 for detector in &package_manager_info.detection.detectors {
    //                     results.push(check(&detector));
    //                 }
    //             }
    //             FrameworkMatchingStrategy::Any => {
    //                 let mut matched = false;
    //                 for detector in &package_manager_info.detection.detectors {
    //                     let result = check(&detector);
    //                     if result {
    //                         matched = result;
    //                         break;
    //                     }
    //                 }
    //
    //                 results.push(matched);
    //             }
    //         }
    //
    //         println!("{:?}", serde_json::to_string(&results));
    //         if  results.iter().all(|&r| r) {
    //             return Some(package_manager_info);
    //         }
    //     }
    //
    //     return None;
    // }

    fn do_install(package_manager: &PackageManagerInfo) -> CifrsResult<ExitStatus> {
        println!("install command {}", &package_manager.install_command);
        let install_command = package_manager.install_command;
        let mut install_process = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", install_command]).spawn()
        } else {
            Command::new("sh").args(["-c", install_command]).spawn()
        }?;

        let status = install_process.wait()?;
        return Ok(status);
    }

    fn do_build(info: &FrameworkInfo) -> CifrsResult<ExitStatus> {
        println!("build command {}", &info.build.command);
        let mut build_process = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", &info.build.command])
                .spawn()
        } else {
            Command::new("sh").args(["-c", &info.build.command]).spawn()
        }?;

        let status = build_process.wait()?;
        return Ok(status);
    }

    pub fn detect_workspace<P: AsRef<Path>>(cwd: P) -> CifrsResult<Option<Workspace>> {
        // TODO: should we try and detect workspace deeper than the current root directory?
        // Vercel uses a max depth of 3 but not sure what use cases that covers.
        let workspaces: Vec<Workspace> = serde_yaml::from_str(WORKSPACES_STR).expect("");

        for workspace in workspaces {
            let m = framework_detection::detect(&cwd, &workspace);
            if m.is_some() {
                return Ok(Some(workspace));
            }
        }

        Ok(None)
    }

    fn get_workspace_package_paths<P: AsRef<Path>>(
        cwd: P,
        workspace: Option<Workspace>,
    ) -> CifrsResult<Vec<PathBuf>> {
        let mut package_paths = HashSet::new();

        if let Some(workspace) = workspace {
            package_paths.extend(workspace.get_package_paths(&cwd)?);
        }

        // make sure root is in
        package_paths.insert(cwd.as_ref().to_path_buf());

        // not uncommon for "docs" to not be part of workspace packages so if its present add it
        let docs_path = PathBuf::from("./docs");
        if docs_path.is_dir() {
            package_paths.insert(docs_path);
        }

        Ok(Vec::from_iter(package_paths))
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use directories::BaseDirs;

    use crate::Cifrs;

    #[test]
    fn check_frameworks() {
        let base_dir = BaseDirs::new().unwrap();
        let home_dir = base_dir.home_dir();
        let framework =
            Cifrs::detect_frameworks(&home_dir.join("workspace/seancarroll.github.io")).unwrap();
        println!("{:?}", framework)
    }

    #[test]
    fn check_workspace() {
        let cwd = env::current_dir().unwrap();
        println!("{:?}", cwd);
        let workspace = Cifrs::detect_workspace(cwd).unwrap();
        println!("{:?}", workspace);
    }

    // TODO: when testing building docs considering adding Netflix's hollow as an example
}
