use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use glob::glob;
use serde_derive::{Deserialize, Serialize};

use crate::framework::{FrameworkDetectionItem, FrameworkDetector, FrameworkMatchingStrategy};
use crate::framework_detection::Detectable;
use crate::projects::project_file::ProjectFile;
use crate::{CifrsError, CifrsResult};

pub const WORKSPACES_STR: &str = include_str!("workspaces.yaml");

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub project_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configs: Vec<PathBuf>,
    pub detection: FrameworkDetector,
}

impl Workspace {
    // TODO: Get workspace package paths
    // TODO: should make an associated enum?
    // TODO: need cwd
    pub fn get_package_paths<P: AsRef<Path>>(&self, cwd: P) -> CifrsResult<Vec<PathBuf>> {
        match self.id.as_str() {
            "cargo" => self.get_cargo_workspace_package_paths(&cwd),
            "msbuild" => self.get_msbuild_solution_workspace_package_paths(cwd),
            "npm" => self.get_package_json_workspace_package_paths(),
            "nx" => self.get_nx_workspace_package_paths(),
            "pnpm" => self.get_pnpm_workspace_package_paths(),
            "rush" => self.get_rush_workspace_package_paths(),
            "yarn" => self.get_package_json_workspace_package_paths(),
            _ => Err(CifrsError::UnknownWorkspaceImplementation(self.id.clone())),
        }
    }

    fn get_cargo_workspace_package_paths<P: AsRef<Path>>(
        &self,
        cwd: P,
    ) -> CifrsResult<Vec<PathBuf>> {
        for project_file in &self.project_files {
            let path = cwd.as_ref().join(project_file.as_str());
            let root: toml::Value = toml::from_str(path.to_str().unwrap())?;
            if let Some(members) = root["workspace"]["members"].as_array() {
                let mut paths = Vec::new();
                for member in members {
                    if let Some(member_str) = member.as_str() {
                        for entry in glob(member_str)
                            .expect("Cargo members should be valid path/glob")
                            .flatten()
                        {
                            paths.push(entry);
                        }
                    }
                }
                return Ok(paths);
            }
        }

        Ok(vec![])
    }

    // getPackageJsonWorkspacePackagePaths
    fn get_package_json_workspace_package_paths(&self) -> CifrsResult<Vec<PathBuf>> {
        for project_file in &self.project_files {
            let root: serde_json::Value = serde_json::from_str(project_file)?;
            if let Some(members) = root["workspaces"].as_array() {
                let mut paths = Vec::new();
                for member in members {
                    if let Some(member_str) = member.as_str() {
                        for entry in glob(member_str)
                            .expect("Workspace should be valid path/glob")
                            .flatten()
                        {
                            paths.push(entry);
                        }
                    }
                }
                return Ok(paths);
            }
        }

        Ok(vec![])
    }

    // getNxWorkspacePackagePaths
    fn get_nx_workspace_package_paths(&self) -> CifrsResult<Vec<PathBuf>> {
        unimplemented!("Figure out how NX workspaces work in without workspace.json given it uses project inference")
    }

    // getPnpmWorkspacePackagePaths
    fn get_pnpm_workspace_package_paths(&self) -> CifrsResult<Vec<PathBuf>> {
        // TODO: this shouldnt use project_files but pnpm-workspace.yaml
        for project_file in &self.project_files {
            let root: serde_yaml::Value = serde_yaml::from_str(project_file)?;
            if let Some(members) = root["packages"].as_sequence() {
                let mut paths = Vec::new();
                for member in members {
                    if let Some(member_str) = member.as_str() {
                        for entry in glob(member_str)
                            .expect("Workspace should be valid path/glob")
                            .flatten()
                        {
                            paths.push(entry);
                        }
                    }
                }
                return Ok(paths);
            }
        }

        Ok(vec![])
    }

    // getRushWorkspacePackagePaths
    fn get_rush_workspace_package_paths(&self) -> CifrsResult<Vec<PathBuf>> {
        // projects
        for project_file in &self.project_files {
            let root: serde_json::Value = serde_json::from_str(project_file)?;
            if let Some(projects) = root["projects"].as_array() {
                let mut paths = Vec::new();
                for project in projects {
                    if let Some(proj) = project.as_object() {
                        if let Some(project_folder) = proj
                            .get("projectFolder")
                            .and_then(|f| f.as_str())
                            .and_then(|f| PathBuf::from_str(f).ok())
                        {
                            paths.push(project_folder);
                        }
                    }
                }
                return Ok(paths);
            }
        }

        Ok(vec![])
    }

    fn get_msbuild_solution_workspace_package_paths<P: AsRef<Path>>(
        &self,
        cwd: P,
    ) -> CifrsResult<Vec<PathBuf>> {
        for entry in fs::read_dir(cwd)?.flatten() {
            if entry.path().to_str().is_some_and(|p| p.contains("*.sln")) {
                let solution_file = MsBuildSolutionFile::parse(entry.path())?;
                return Ok(solution_file.project_paths);
            }
        }

        Ok(vec![])
    }
}

impl Detectable for Workspace {
    fn get_matching_strategy(&self) -> &FrameworkMatchingStrategy {
        &self.detection.matching_strategy
    }

    fn get_detectors(&self) -> &Vec<FrameworkDetectionItem> {
        &self.detection.detectors
    }

    // For Frameworks this should return &Vec but this looks to returning an owned type
    // Maybe this calls for a Cow?
    fn get_project_files(&self) -> Cow<Vec<ProjectFile>> {
        Cow::Owned(
            self.project_files
                .iter()
                .filter_map(|p| ProjectFile::from_path(p).ok())
                .collect::<Vec<ProjectFile>>(),
        )
    }

    fn get_configuration_files(&self) -> &Vec<PathBuf> {
        &self.configs
    }
}

struct MsBuildSolutionFile {
    pub project_paths: Vec<PathBuf>,
}

/// A solution is a structure for organizing projects in Visual Studio and maintains state in a
/// text-based solution, .sln, file.
impl MsBuildSolutionFile {
    /// Solution files contain project declarations of the format:
    /// Project("{Project-Type-GUID}") = "Project-Name", "Project-Path.extension", "{Project-GUID}"
    fn parse<P: AsRef<Path>>(path: P) -> CifrsResult<Self> {
        let content = fs::read_to_string(path)?;

        let mut project_paths = Vec::new();
        for line in content.lines() {
            if line.starts_with("Project") {
                let project_parts = line.split(",").collect::<Vec<&str>>();
                if project_parts.len() == 3 {
                    if let Ok(project_file) = PathBuf::from_str(project_parts[1]) {
                        if project_file.is_file() {
                            if let Some(project_path) = project_file.parent() {
                                project_paths.push(project_path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }

        Ok(MsBuildSolutionFile { project_paths })
    }
}

#[cfg(test)]
mod tests {
    use crate::workspaces::{Workspace, WORKSPACES_STR};

    #[test]
    fn test_deserialize_workspace_yaml() {
        let workspaces: Vec<Workspace> = serde_yaml::from_str(WORKSPACES_STR).expect("");
        println!("{}", serde_json::to_string(&workspaces).unwrap());
    }
}
