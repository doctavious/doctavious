use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use glob::PatternError;
use thiserror::Error;
use tracing::error;

use crate::framework_detection::Detectable;
use crate::frameworks::{FrameworkBuildArgs, FrameworkInfo};
use crate::package_manager::PackageManagerInfo;
use crate::workspaces::Workspace;

mod backends;
mod framework_detection;
mod frameworks;
mod js_module;
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

    #[error("Unknown framework format: {0}")]
    UnknownFrameworkFormat(String),

    #[error("Unknown project file: {0}")]
    UnknownProjectFilePath(String),

    #[error("Unknown workspace implementation: {0}")]
    UnknownWorkspaceImplementation(String),
}

pub type CifrsResult<T> = Result<T, CifrsError>;

pub struct Cifrs;

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
            for framework in frameworks::get_all() {
                let m = framework_detection::detect(&dir, &framework);
                // TODO: return MatchResult?
                if m.is_some() {
                    return Ok(framework);
                }
            }
        }

        Err(CifrsError::MissingFrameworkConfig())
    }

    pub fn build<P: AsRef<Path>>(path: &P, install: bool) -> CifrsResult<()> {
        let framework = Cifrs::detect_frameworks(path)?;

        if install {
            let package_manager_info = Cifrs::detect_framework_package_manager(&path, &framework);
            // TODO(Sean): if we can't get package_manager should this fail?
            if let Some(package_manager_info) = package_manager_info {
                let install_status = Cifrs::do_install(&package_manager_info)?;
                if !install_status.success() {
                    let install_status_code = install_status
                        .code()
                        .map_or("unknown".to_string(), |s| s.to_string());
                    println!("install failed with code: {:?}", install_status_code);
                    return Err(CifrsError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("install failed with code: {:?}", install_status_code),
                    )));
                }
            }
        }

        let build_status =
            Cifrs::do_build(&framework.build.command, &framework.build.command_args)?;
        if !build_status.success() {
            let build_status_code = build_status
                .code()
                .map_or("unknown".to_string(), |s| s.to_string());
            println!("build failed with code: {:?}", build_status_code);
            return Err(CifrsError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("build failed with code: {:?}", build_status_code),
            )));
        }

        Ok(())
    }

    fn detect_package_manager<P: AsRef<Path>>(cwd: P) -> Option<PackageManagerInfo> {
        for pm in package_manager::get_list() {
            let m = framework_detection::detect(&cwd, &pm);
            if m.is_some() {
                return Some(pm);
            }
        }
        None
    }

    fn detect_framework_package_manager<P: AsRef<Path>>(
        cwd: P,
        framework: &FrameworkInfo,
    ) -> Option<PackageManagerInfo> {
        for p in framework.backend.project_files() {
            for pm in p.supported_package_managers() {
                let m = framework_detection::detect(&cwd, &pm);
                if m.is_some() {
                    return Some(pm);
                }
            }
        }

        None
    }

    fn do_install(package_manager: &PackageManagerInfo) -> CifrsResult<ExitStatus> {
        Ok(Cifrs::get_command(package_manager.install_command.as_str())
            .spawn()?
            .wait()?)
    }

    fn do_build(cmd: &str, args: &Option<FrameworkBuildArgs>) -> CifrsResult<ExitStatus> {
        let mut command = Cifrs::get_command(cmd);

        // TODO(Sean): pass appropriate values to command
        if let Some(args) = args {
            if let Some(config) = &args.config {}

            if let Some(output) = &args.output {}

            if let Some(source) = &args.source {}
        }

        Ok(command.spawn()?.wait()?)
    }

    fn get_command(cmd: &str) -> Command {
        let mut command = if cfg!(target_os = "windows") {
            Command::new("cmd")
        } else {
            Command::new("sh")
        };

        if cfg!(target_os = "windows") {
            command.args(["/C", cmd])
        } else {
            command.args(["-c", cmd])
        };

        command
    }

    pub fn detect_workspace<'a, P: AsRef<Path>>(cwd: P) -> CifrsResult<Option<Workspace>> {
        // TODO: should we try and detect workspace deeper than the current root directory?
        // Vercel uses a max depth of 3 but not sure what use cases that covers.
        for workspace in workspaces::get_all() {
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

    // TODO: when testing building docs considering adding Netflix's hollow as an example
    #[test]
    fn check_hollow() {
        let base_dir = BaseDirs::new().unwrap();
        let home_dir = base_dir.home_dir();
        let framework =
            Cifrs::detect_frameworks(&home_dir.join("workspace/netflix/hollow")).unwrap();
        println!("{:?}", framework)
    }

    #[test]
    fn check_workspace() {
        let cwd = env::current_dir().unwrap();
        println!("{:?}", cwd);
        let workspace = Cifrs::detect_workspace(cwd).unwrap();
        println!("{:?}", workspace);
    }

    #[test]
    fn check_package_manager() {
        let cwd = env::current_dir().unwrap();
        println!("{:?}", cwd);
        let package_manager = Cifrs::detect_package_manager(cwd).unwrap();
        println!("{:?}", package_manager);
    }

    #[test]
    fn check_site_package_manager() {
        let base_dir = BaseDirs::new().unwrap();
        let home_dir = base_dir.home_dir();
        let package_manager =
            Cifrs::detect_package_manager(&home_dir.join("workspace/seancarroll.github.io"))
                .unwrap();
        println!("{:?}", package_manager)
    }
}
