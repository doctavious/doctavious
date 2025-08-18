use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error};

use crate::package_manager::PackageManagerInfo;
// use crate::package_manager::PackageManager;
use crate::projects::msbuild::MsBuildProj;
use crate::projects::project_file::ProjectFile::RequirementsTxt;
use crate::{CifrsError, CifrsResult};

// TODO: lets create a projects module and put this along side CSProj given their relationship
// I think we should put them closer in proximity

// This would allow us to split existence from dependency
// ProjectFile
// path
// type
// content

lazy_static! {
    static ref MSBUILD_PROJECT_FILE_PATTERNS: Vec<Regex> = vec![
        Regex::new(".*.csproj").unwrap(),
        Regex::new(".*.fsproj").unwrap()
    ];
}

pub struct Proj {
    pub path: PathBuf,
    pub project_type: ProjectFile,
    pub content: String,
}

impl Proj {
    // pub(crate) fn new(
    //     path: PathBuf,
    //     project_type: ProjectFile,
    // ) -> DoctaviousResult<Proj> {
    //     let content = fs::read_to_string(path)?;
    //     Ok(Self {
    //         path: path.clone(),
    //         project_type,
    //         content
    //     })
    // }

    // pub(crate) fn has_dependency(&self, name: &'static str) -> bool {
    //     match self.project_type {
    //         ProjectFile::CargoToml => {
    //             let root: toml::Value = toml::from_str(self.content.as_str())?;
    //             // TODO: do we want to check dev-packages
    //             root.get("dependencies")
    //                 .and_then(|o| o.get(name))
    //                 .is_some()
    //         }
    //         ProjectFile::CSProj => {
    //             let mut has_dependency = false;
    //             let result: Result<CSProj, _> = serde_xml_rs::from_str(content.as_str());
    //             if let Ok(build_proj) = result {
    //                 for item_group in build_proj.item_groups {
    //                     // could also do item_group.package_references.unwrap_or_default()
    //                     if let Some(package_references ) = item_group.package_references {
    //                         for pkref in package_references {
    //                             if name == pkref.include {
    //                                 has_dependency = true;
    //                                 break;
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //
    //             has_dependency
    //         }
    //         ProjectFile::GemFile => {
    //             self.content.contains(&format!("gem '{}'", name))
    //         }
    //         ProjectFile::GoMod => {
    //             self.content.contains(&format!("{}", name))
    //         }
    //         ProjectFile::PackageJson => {
    //             let root: Value = serde_json::from_str(self.content.as_str())?;
    //             // TODO: do we want to check devDependencies
    //             root.get("dependencies")
    //                 .and_then(|o| o.get(name))
    //                 .is_some()
    //         }
    //         ProjectFile::PipFile => {
    //             let root: toml::Value = toml::from_str(self.content.as_str())?;
    //             // TODO: do we want to check dev-packages
    //             root.get("packages")
    //                 .and_then(|o| o.get(name))
    //                 .is_some()
    //         }
    //         ProjectFile::PyProject => {
    //             let root: toml::Value = toml::from_str(self.content.as_str())?;
    //             // might be to do these individual lookup
    //             root.get("tool.poetry.dependencies")
    //                 .and_then(|o| o.get(name))
    //                 .is_some()
    //         }
    //         ProjectFile::RequirementsTxt => {
    //             self.content.contains(&format!("{}==", name))
    //         }
    //     }
    // }
}

// impl would have a get_project_files() -> Vec<ProjectFiles>

// Manifest / ManifestFile
// ProjectFileType
// ProjectType
// SpecFile
#[non_exhaustive]
#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectFile {
    CargoToml,
    GemFile,
    GoMod,
    MsBuild,
    PackageJson,
    PipFile,
    PyProject,
    RequirementsTxt,
}

impl ProjectFile {
    pub fn from_path<S: AsRef<str>>(s: S) -> CifrsResult<Self> {
        let path = s.as_ref();
        for pattern in MSBUILD_PROJECT_FILE_PATTERNS.deref() {
            if pattern.is_match(path) {
                return Ok(ProjectFile::MsBuild);
            }
        }

        match path {
            "go.mod" => Ok(ProjectFile::GoMod),
            "package.json" => Ok(ProjectFile::PackageJson),
            "pipfile" => Ok(ProjectFile::PipFile),
            "pyproject.toml" => Ok(ProjectFile::PyProject),
            "requirements.txt" => Ok(RequirementsTxt),
            "Gemfile" => Ok(ProjectFile::GemFile),
            "Cargo.toml" => Ok(ProjectFile::CargoToml),
            _ => Err(CifrsError::UnknownProjectFilePath(path.to_string())),
        }
    }
}

impl ProjectFile {
    // TODO: should return option?
    pub fn get_project_path<P: AsRef<Path>>(&self, cwd: P) -> PathBuf {
        let dir = cwd.as_ref();
        match self {
            Self::CargoToml => dir.join("Cargo.toml"),
            Self::GemFile => dir.join("Gemfile"),
            Self::GoMod => dir.join("go.mod"),
            Self::MsBuild => {
                for entry in fs::read_dir(&cwd).unwrap().flatten() {
                    // TODO: improve this. Should come from project
                    if entry
                        .path()
                        .extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|s| ["csproj", "fsproj"].contains(&s))
                    {
                        return entry.path();
                    }
                }

                dir.join(".csproj")
            }
            Self::PackageJson => dir.join("package.json"),
            Self::PipFile => dir.join("pipfile"),
            Self::PyProject => dir.join("pyproject.toml"),
            Self::RequirementsTxt => dir.join("requirements.txt"),
        }
    }

    // pub fn supported_package_managers(&self) -> &[PackageManager] {
    //     match self {
    //         Self::CargoToml => &[PackageManager::Cargo],
    //         Self::GemFile => &[PackageManager::Bundler],
    //         Self::GoMod => &[PackageManager::GoModules],
    //         Self::MsBuild => &[PackageManager::Nuget],
    //         Self::PackageJson => &[
    //             PackageManager::Npm,
    //             PackageManager::Pnpm,
    //             PackageManager::Yarn,
    //         ],
    //         Self::PipFile => &[PackageManager::Pip],
    //         Self::PyProject => &[PackageManager::Pip, PackageManager::Poetry],
    //         Self::RequirementsTxt => &[PackageManager::Pip],
    //     }
    // }

    pub fn supported_package_managers(&self) -> Vec<PackageManagerInfo> {
        match self {
            Self::CargoToml => PackageManagerInfo::find_by_ids(vec!["cargo"]),
            Self::GemFile => PackageManagerInfo::find_by_ids(vec!["bundler"]),
            Self::GoMod => PackageManagerInfo::find_by_ids(vec!["go"]),
            // TODO: should be nuget? probably should look at packet?
            Self::MsBuild => PackageManagerInfo::find_by_ids(vec!["nuget"]),
            Self::PackageJson => PackageManagerInfo::find_by_ids(vec!["npm", "pnpm", "yarn"]),
            Self::PipFile => PackageManagerInfo::find_by_ids(vec!["pip"]),
            Self::PyProject => PackageManagerInfo::find_by_ids(vec!["pip", "poetry"]),
            Self::RequirementsTxt => PackageManagerInfo::find_by_ids(vec!["pip"]),
        }
    }

    pub fn has_dependency<P: AsRef<Path>>(&self, cwd: P, dependency: &str) -> CifrsResult<bool> {
        let project_file_path = self.get_project_path(&cwd);
        if !project_file_path.is_file() {
            debug!("Project file {:?} not found...skipping", &project_file_path);
        }

        match fs::read_to_string(&project_file_path) {
            Ok(project_file_content) => {
                let found = match self {
                    Self::CargoToml => {
                        let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                        root["dependencies"][dependency].is_str()
                            || root["dev-dependencies"][dependency].is_str()
                    }
                    Self::GemFile => project_file_content.contains(&format!("gem '{dependency}'")),
                    Self::GoMod => project_file_content.contains(dependency),
                    Self::MsBuild => {
                        let mut has_dependency = false;
                        match serde_xml_rs::from_str::<MsBuildProj>(&project_file_content) {
                            Ok(build_proj) => {
                                if build_proj.has_package_reference(dependency) {
                                    has_dependency = true;
                                }
                            }
                            Err(e) => {
                                error!("{}", e);
                            }
                        }

                        has_dependency
                    }
                    Self::PackageJson => {
                        let root: Value = serde_json::from_str(project_file_content.as_str())?;
                        !root["dependencies"][dependency].is_null()
                            || !root["devDependencies"][dependency].is_null()
                    }
                    Self::PipFile => {
                        let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                        root["packages"][dependency].is_str()
                            || root["dev-packages"][dependency].is_str()
                    }
                    Self::PyProject => {
                        let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                        // TODO: improve this...support more
                        if let Some(tool) = root.get("tool") {
                            tool.get("poetry")
                                .and_then(|p| p.get("dependencies"))
                                .and_then(|dependencies| dependencies.get(dependency))
                                .is_some_and(|dependency| dependency.is_str())
                        } else if let Some(project) = root.get("project") {
                            // UV structure
                            if let Some(dependencies) = project.get("dependencies") {
                                if let Some(dep_array) = dependencies.as_array() {
                                    dep_array.iter().any(|dep| {
                                        if let Some(dep_str) = dep.as_str() {
                                            // Strip at first occurrence of <, >, =, ;
                                            let mut cutoff = dep_str.len();
                                            for sep in ['<', '>', '=', ';'] {
                                                if let Some(pos) = dep_str.find(sep) {
                                                    cutoff = cutoff.min(pos);
                                                }
                                            }
                                            let name = dep_str[..cutoff].trim().to_string();
                                            name == dependency
                                        } else {
                                            false
                                        }
                                    })
                                } else {
                                    // TODO: log unexpected dependencies value. expected array
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            // TODO: log that we couldnt parse and to let us known to support...
                            false
                        }
                    }
                    RequirementsTxt => project_file_content
                        .lines()
                        .find(|l| l.trim().starts_with(dependency))
                        .is_some(),
                };

                if found {
                    return Ok(true);
                }
            }
            Err(e) => {
                // error!("Error reading project file: {:?}", &project_file_path);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use crate::projects::project_file::ProjectFile;

    #[test]
    fn test_pyproject_poetry() {
        let tmp_dir = TempDir::new().unwrap();
        fs::copy(
            "./tests/fixtures/projects/python/pyproject_poetry.toml",
            tmp_dir.path().join("pyproject.toml"),
        )
        .unwrap();

        let found = ProjectFile::PyProject
            .has_dependency(&tmp_dir, "python")
            .unwrap();
        assert!(found);
    }

    #[test]
    fn test_pyproject_uv() {
        let tmp_dir = TempDir::new().unwrap();
        fs::copy(
            "./tests/fixtures/projects/python/pyproject_uv.toml",
            tmp_dir.path().join("pyproject.toml"),
        )
        .unwrap();

        let found = ProjectFile::PyProject
            .has_dependency(&tmp_dir, "requests")
            .unwrap();
        assert!(found);
    }

    #[test]
    fn test_msbuild_proj() {
        let tmp_dir = TempDir::new().unwrap();
        fs::copy(
            "./tests/fixtures/projects/msbuild/docs.csproj",
            tmp_dir.path().join("docs.csproj"),
        )
        .unwrap();

        let found = ProjectFile::MsBuild
            .has_dependency(&tmp_dir, "Microsoft.Orleans.Server")
            .unwrap();
        assert!(found);
    }
}
