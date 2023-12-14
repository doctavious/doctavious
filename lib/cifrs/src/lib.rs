use std::collections::HashSet;
use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::str::FromStr;

use glob::PatternError;
use thiserror::Error;
use tracing::error;

use crate::framework_detection::Detectable;
use crate::frameworks::{FrameworkBuildArg, FrameworkBuildArgs, FrameworkInfo};
use crate::package_manager::PackageManagerInfo;
use crate::workspaces::Workspace;

mod backends;
mod framework_detection;
pub mod frameworks;
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

pub trait PrintCommand {
    fn print_command(&self) -> String;
}

impl PrintCommand for Command {
    fn print_command(&self) -> String {
        let mut cmd_parts = vec![self.get_program().to_str()];
        for arg in self.get_args() {
            cmd_parts.push(arg.to_str())
        }

        cmd_parts
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect::<Vec<&str>>()
            .join(" ")
    }
}

// TODO: Not sure how i feel about this. We need to return output path when we actually run the
// build command but dont ned it for dryrun. This could also allow us to return stages/steps
// that are run so that we can generate appropriate logs/output for dryrun
pub enum BuildOutput {
    DryRun,
    Invoked(BuildResult),
}

pub struct BuildResult {
    pub dir: PathBuf,
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

    // TODO: should we return more than one?
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

    // TODO: add optional config
    // TODO: add optional destination
    // TODO: instead of install boolean include a install_override which is optional string.
    // we would generally perform an install but it could be avoided by an empty string
    // essentially this needs to support 2 scenarios
    // 1. CLI - Build project locally.
    //  - should we get project settings from project settings if project is linked?
    // 2. Server (webhook from SCM) - This would be configured via website project settings
    // Website project settings include
    //  - build command
    //  - output directory
    //  - install command
    //  - ignore-build-command - ex: "git diff --quiet HEAD^ HEAD ./". (not applicable to CLI)
    // TODO: should we force output to `.doctavious/output`?
    pub fn build<P: AsRef<Path>>(
        path: &P,
        dry: bool,
        install: bool,
        // config_override: Option<PathBuf>,
        // output_override: Option<PathBuf>,
    ) -> CifrsResult<BuildOutput> {
        let framework = Cifrs::detect_frameworks(path)?;

        if install {
            let package_manager_info = Cifrs::detect_framework_package_manager(&path, &framework);
            // TODO(Sean): if we can't get package_manager should this fail?
            if let Some(package_manager_info) = package_manager_info {
                let mut install_command =
                    Cifrs::get_command(package_manager_info.install_command.as_str());

                if dry {
                    println!("{}", install_command.print_command());
                } else {
                    let install_status = install_command.spawn()?.wait()?;
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
        }

        let mut build_command = Cifrs::get_command(&framework.build.command);
        println!("{:?}", serde_json::to_string(&framework));
        if let Some(args) = &framework.build.command_args {
            let mut build_args = vec![];
            if let Some(config) = &args.config {
                if let Some(framework_config) = framework.get_configuration() {
                    match config {
                        FrameworkBuildArg::Arg { index, .. } => {
                            build_args
                                .push((index, framework_config.path.to_string_lossy().to_string()));
                        }
                        FrameworkBuildArg::Option { name, .. } => {
                            build_command
                                .arg(name)
                                .arg(framework_config.path.to_string_lossy().to_string());
                        }
                    }
                }
            }

            if let Some(output) = &args.output {
                match output {
                    FrameworkBuildArg::Arg { index, .. } => {
                        build_args.push((index, framework.build.output_directory.to_string()));
                    }
                    FrameworkBuildArg::Option { name, .. } => {
                        build_command
                            .arg(name)
                            .arg(framework.build.output_directory.to_string());
                    }
                }
            }

            if let Some(source) = &args.source {
                let source_path = path.as_ref().to_string_lossy().to_string();
                match source {
                    FrameworkBuildArg::Arg { index, .. } => {
                        build_args.push((index, source_path));
                    }
                    FrameworkBuildArg::Option { name, .. } => {
                        build_command.arg(name).arg(source_path);
                    }
                }
            }

            build_args.sort_by_key(|a| a.0);
            for (_, value) in build_args {
                build_command.arg(value);
            }
        }

        if dry {
            println!("{}", build_command.print_command());
        } else {
            println!("{}", build_command.print_command());
            let build_status = build_command.spawn()?.wait()?;
            // let build_status = build_command.output()?;
            println!("{:?}", &build_status);
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
        }

        if dry {
            Ok(BuildOutput::DryRun)
        } else {
            Ok(BuildOutput::Invoked(BuildResult {
                dir: PathBuf::from(framework.build.output_directory),
            }))
        }
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

    fn get_command(cmd: &str) -> Command {
        // let mut command = if cfg!(target_os = "windows") {
        //     Command::new("cmd")
        // } else {
        //     Command::new("sh")
        // };
        //
        // if cfg!(target_os = "windows") {
        //     command.args(["/C", cmd])
        // } else {
        //     command.args(["-c", cmd])
        // };
        let program_args = cmd.split(" ").collect::<Vec<&str>>();
        let mut command = Command::new(program_args[0]);
        if program_args.len() > 1 {
            command.args(&program_args[1..]);
        }
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

    // TODO: replace with local repos
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
