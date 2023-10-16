use std::path::PathBuf;
use std::{env, fs};

use glob::glob;
use serde_derive::Serialize;
use serde_json::Value;

use crate::package_manager::PackageManager;
use crate::projects::csproj::CSProj;
use crate::CifrsResult;

// TODO: lets create a projects module and put this along side CSProj given their relationship
// I think we should put them closer in proximity

// This would allow us to split existence from dependency
// ProjectFile
// path
// type
// content

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

// Manifest
// ProjectFileType
// ProjectType
#[non_exhaustive]
#[derive(Clone, Copy, Serialize)]
pub enum ProjectFile {
    CargoToml,
    CSProj,
    GemFile,
    GoMod,
    PackageJson,
    PipFile,
    PyProject,
    RequirementsTxt,
}

// random thought
// pub struct ProjectFile<T> {
//     pub path: String,
//     pub raw_content: String,
//     pub content: Option<T>
// }

impl ProjectFile {
    // pub fn file_name(&self) -> &str {
    //     match self {
    //         ProjectFile::GemFile => "Gemfile",
    //         ProjectFile::PackageJson => "package.json",
    //         ProjectFile::PipFile => "pipfile",
    //         ProjectFile::PyProject => "pyproject.toml",
    //         ProjectFile::RequirementsTxt => "requirements.txt"
    //     }
    // }

    pub fn get_project_paths(&self) -> Vec<PathBuf> {
        match self {
            ProjectFile::CSProj => {
                let glob_result = glob("**/*.csproj");
                match glob_result {
                    Ok(paths) => paths.into_iter().filter_map(|p| p.ok()).collect(),
                    Err(e) => {
                        // TODO: log
                        vec![]
                    }
                }
            }
            ProjectFile::GoMod => vec![PathBuf::from("go.mod")],
            ProjectFile::PackageJson => vec![PathBuf::from("package.json")],
            ProjectFile::PipFile => vec![PathBuf::from("pipfile")],
            ProjectFile::PyProject => vec![PathBuf::from("pyproject.toml")],
            ProjectFile::RequirementsTxt => vec![PathBuf::from("requirements.txt")],
            ProjectFile::GemFile => vec![PathBuf::from("Gemfile")],
            ProjectFile::CargoToml => vec![PathBuf::from("cargo.toml")],
        }
    }

    pub fn supported_package_managers(&self) -> &[PackageManager] {
        match self {
            ProjectFile::CargoToml => &[PackageManager::Cargo],
            ProjectFile::CSProj => &[PackageManager::Nuget],
            ProjectFile::GemFile => &[PackageManager::Bundler],
            ProjectFile::GoMod => &[PackageManager::Go],
            ProjectFile::PackageJson => &[
                PackageManager::Npm,
                PackageManager::Pnpm,
                PackageManager::Yarn,
            ],
            ProjectFile::PipFile => &[PackageManager::Pip],
            ProjectFile::PyProject => &[PackageManager::Pip, PackageManager::Poetry],
            ProjectFile::RequirementsTxt => &[PackageManager::Pip],
        }
    }

    pub fn has_dependency(&self, dependency: &str) -> CifrsResult<bool> {
        // let project_file_content = fs::read_to_string(self.file_name())?;
        let found = match self {
            ProjectFile::CargoToml => {
                let project_file_content = fs::read_to_string("cargo.toml")?;
                let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                // TODO: do we want to check dev-packages
                root.get("dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::CSProj => {
                println!("hello there...");
                let mut has_dependency = false;
                match env::current_dir() {
                    Ok(p) => {
                        println!("path...{:?}", p);
                    }
                    Err(e) => {
                        println!("error getting cwd...{}", e);
                    }
                }

                let paths = fs::read_dir("./").unwrap();
                for path in paths {
                    println!("path name: {}", path.unwrap().path().display())
                }
                for file in glob("*")? {
                    println!("path...{:?}", file.unwrap());
                }

                for entry in glob("**/*.csproj")?.flatten() {
                    if let Some(path_str) = entry.to_str() {
                        let content = fs::read_to_string(path_str)?;
                        let result: Result<CSProj, _> = serde_xml_rs::from_str(content.as_str());
                        // match result {
                        //     Ok(r) => {
                        //         println!("got project...{:?}", r);
                        //     }
                        //     Err(e) => {
                        //         println!("serde error...{:?}", e);
                        //     }
                        // }
                        if let Ok(build_proj) = result {
                            println!("build proj...{:?}", build_proj);
                            for item_group in build_proj.item_groups {
                                // could also do item_group.package_references.unwrap_or_default()
                                if let Some(package_references) = item_group.package_references {
                                    for pkref in package_references {
                                        if dependency == pkref.include {
                                            has_dependency = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                has_dependency
            }
            ProjectFile::GemFile => {
                let project_file_content = fs::read_to_string("Gemfile")?;
                project_file_content.contains(&format!("gem '{}'", dependency))
            }
            ProjectFile::GoMod => {
                todo!("implement")
            }
            ProjectFile::PackageJson => {
                let project_file_content = fs::read_to_string("package.json")?;
                let root: Value = serde_json::from_str(project_file_content.as_str())?;
                // TODO: do we want to check devDependencies
                root.get("dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::PipFile => {
                let project_file_content = fs::read_to_string("pipfile")?;
                let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                // TODO: do we want to check dev-packages
                root.get("packages")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::PyProject => {
                let project_file_content = fs::read_to_string("pyproject.toml")?;
                let root: toml::Value = toml::from_str(project_file_content.as_str())?;
                // might be to do these individual lookup
                root.get("tool.poetry.dependencies")
                    .and_then(|o| o.get(dependency))
                    .is_some()
            }
            ProjectFile::RequirementsTxt => {
                let project_file_content = fs::read_to_string("requirements.txt")?;
                project_file_content.contains(&format!("{}==", dependency))
            }
        };

        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    // use crate::commands::build::package_manager::ProjectFile;
    use std::io::Write;
    use std::panic::{RefUnwindSafe, UnwindSafe};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::{env, fs, io, panic};

    use once_cell::sync::Lazy;
    use serial_test::serial;
    use swc::atoms::once_cell;
    use tempfile::TempDir;

    use crate::projects::project_file::ProjectFile;
    use crate::CifrsResult;
    // use crate::commands::build::project_file::ProjectFile;

    static SERIAL_TEST: Lazy<Mutex<()>> = Lazy::new(Default::default);

    #[test]
    fn test_pyproject() {
        let content = r#"
[tool.poetry]
name = "poetry-demo"
version = "0.1.0"
description = ""
authors = ["Sébastien Eustace <sebastien@eustace.io>"]
readme = "README.md"
packages = [{include = "poetry_demo"}]

[tool.poetry.dependencies]
python = "^3.7"
        "#;
    }

    #[test]
    #[serial]
    fn test_csproj() -> CifrsResult<()> {
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net7.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <OutputType>Exe</OutputType>
    <ServerGarbageCollection>true</ServerGarbageCollection>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.Extensions.Hosting" Version="7.0.0" />
    <PackageReference Include="Microsoft.Extensions.Logging.Console" Version="7.0.0" />
    <PackageReference Include="Microsoft.Orleans.Server" Version="7.0.0" />
    <PackageReference Include="Microsoft.Orleans.Streaming" Version="7.0.0" />
  </ItemGroup>

  <ItemGroup>
    <ProjectReference Include="..\ChatRoom.Common\ChatRoom.Common.csproj" />
  </ItemGroup>

</Project>"#;

        let tmp_dir = TempDir::new().unwrap();
        let file_path = tmp_dir.path().join("docs.csproj");
        let mut tmp_file = File::create(file_path).unwrap();
        writeln!(tmp_file, "{}", content)?;

        // println!("{:?}", tmp_dir);
        // println!("cur dir is  {:?}", env::current_dir()?);
        let a = with_dir(&tmp_dir, || {
            match env::current_dir() {
                Ok(p) => {
                    println!("path...{:?}", p);
                }
                Err(e) => {
                    println!("error with cwd...{}", e);
                }
            }

            let paths = fs::read_dir(&tmp_dir).unwrap();
            for path in paths {
                println!("path name: {}", path.unwrap().path().display())
            }

            let found = ProjectFile::CSProj
                .has_dependency("Microsoft.Orleans.Server")
                .unwrap();
            println!("dependency found: {}", found);
            assert!(found);
            Ok(())
        });

        match a {
            Ok(_) => {}
            Err(e) => {
                println!("error was {}", e);
            }
        }

        // let found = ProjectFile::CSProj.has_dependency("Microsoft.Orleans.Server")?;
        // assert!(found);

        Ok(())
    }

    // with_directory(path, || { closure })

    pub fn with_dir<P, F, R>(path: &P, closure: F) -> io::Result<R>
    where
        P: AsRef<Path>,
        F: Fn() -> io::Result<R> + UnwindSafe + RefUnwindSafe,
    {
        let guard = SERIAL_TEST.lock().unwrap();
        let original_dir = env::current_dir()?;
        match env::set_current_dir(path) {
            Ok(_) => {
                println!("success");
            }
            Err(e) => {
                println!("error...{:?}", e);
            }
        }

        // println!("current...{:?}", env::current_dir()?);
        let a = match panic::catch_unwind(closure) {
            Ok(result) => {
                println!("original dir...{:?}", original_dir);
                env::set_current_dir(original_dir)?;
                // drop(path); // not sure if we need do drop this here
                result
            }
            Err(err) => {
                println!("error occurred original dir...{:?}", original_dir);
                env::set_current_dir(original_dir)?;
                // drop(path);
                drop(guard);
                panic::resume_unwind(err);
            }
        };
        a
    }
}
