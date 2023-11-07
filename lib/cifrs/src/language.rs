use serde_derive::{Deserialize, Serialize};

use crate::package_manager::PackageManager;
use crate::projects::project_file::ProjectFile;

// TODO: We might need to determine python path in order to do python builds

#[non_exhaustive]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum Language {
    CSharp,
    Go,
    Javascript,
    Python,
    Ruby,
    Rust,
}

impl Language {
    // pub fn get_projects(&self) -> Vec<Proj> {
    //     match self {
    //         Language::DotNet => {
    //             let glob_result = glob("**/*.csproj");
    //             match glob_result {
    //                 Ok(paths) => {
    //                     let mut projects = Vec::new();
    //                     for path in paths {
    //                         if let Ok(path) = path {
    //                             let project = Proj::new(path, ProjectFile::CSProj);
    //                             match project {
    //                                 Ok(p) => projects.push(p),
    //                                 Err(_) => {
    //                                     // TODO: print unable to read path
    //                                 }
    //                             }
    //
    //                         } else {
    //                             // TODO: log
    //                         }
    //                     }
    //                     projects
    //                 },
    //                 Err(e) => {
    //                     // TODO: log
    //                     vec![]
    //                 }
    //             }
    //         },
    //         Language::Go => vec![Proj::new(PathBuf::from("go.mod"), ProjectFile::GoMod)],
    //         Language::Javascript => vec![Proj::new(PathBuf::from("package.json"), ProjectFile::PackageJson],
    //         Language::Python => vec![
    //             PathBuf::from("pipfile"),
    //             PathBuf::from("pyproject.toml"),
    //             PathBuf::from("requirements.txt")
    //         ],
    //         Language::Ruby => vec![PathBuf::from("Gemfile")],
    //         Language::Rust => vec![PathBuf::from("cargo.toml")]
    //     }
    // }

    // pub fn get_project_paths(&self) -> Vec<PathBuf> {
    //     match self {
    //         Language::DotNet => {
    //             let glob_result = glob("**/*.csproj");
    //             match glob_result {
    //                 Ok(paths) => {
    //                     paths.into_iter().filter_map(|p| p.ok()).collect()
    //                 },
    //                 Err(e) => {
    //                     // TODO: log
    //                     vec![]
    //                 }
    //             }
    //         },
    //         Language::Go => vec![PathBuf::from("go.mod")],
    //         Language::Javascript => vec![PathBuf::from("package.json")],
    //         Language::Python => vec![
    //             PathBuf::from("pipfile"),
    //             PathBuf::from("pyproject.toml"),
    //             PathBuf::from("requirements.txt")
    //         ],
    //         Language::Ruby => vec![PathBuf::from("Gemfile")],
    //         Language::Rust => vec![PathBuf::from("cargo.toml")]
    //     }
    // }

    // pub fn get_project_files(&self) -> Vec<Proj> {
    //     match self {
    //         Language::DotNet => {
    //             let project_file_content = fs::read_to_string("cargo.toml")?;
    //         },
    //         Language::Go => &[ProjectFile::GoMod],
    //         Language::Javascript => &[ProjectFile::PackageJson],
    //         Language::Python => &[ProjectFile::PyProject, ProjectFile::PipFile, ProjectFile::RequirementsTxt],
    //         Language::Ruby => &[ProjectFile::GemFile],
    //         Language::Rust => &[ProjectFile::CargoToml]
    //     }
    // }

    // pub const fn project_files(&self) -> &[ProjectFile] {
    //     match self {
    //         Language::CSharp => &[ProjectFile::CSProj],
    //         // F# has .fsproj
    //         Language::Go => &[ProjectFile::GoMod],
    //         Language::Javascript => &[ProjectFile::PackageJson],
    //         Language::Python => &[
    //             ProjectFile::PyProject,
    //             ProjectFile::PipFile,
    //             ProjectFile::RequirementsTxt,
    //         ],
    //         Language::Ruby => &[ProjectFile::GemFile],
    //         Language::Rust => &[ProjectFile::CargoToml],
    //     }
    // }

    pub const fn get_package_managers(&self) -> &[PackageManager] {
        match self {
            Language::CSharp => &[PackageManager::Nuget],
            Language::Go => &[PackageManager::GoModules],
            Language::Javascript => &[
                PackageManager::Npm,
                PackageManager::Pnpm,
                PackageManager::Yarn,
            ],
            Language::Python => &[PackageManager::Poetry, PackageManager::Pip],
            Language::Ruby => &[PackageManager::Bundler],
            Language::Rust => &[PackageManager::Cargo],
        }
    }

    pub fn info(&self) -> LanguageInfo {
        match self {
            // F# supports paket
            Language::CSharp => LanguageInfo {
                name: "C#",
                package_managers: SupportedPackageManagers {
                    supported: vec![PackageManager::Nuget],
                    fallback: PackageManager::Nuget,
                },
            },
            Language::Go => LanguageInfo {
                name: "Go",
                package_managers: SupportedPackageManagers {
                    supported: vec![PackageManager::GoModules],
                    fallback: PackageManager::GoModules,
                },
            },
            Language::Javascript => LanguageInfo {
                name: "JavaScript",
                package_managers: SupportedPackageManagers {
                    supported: vec![
                        PackageManager::Npm,
                        PackageManager::Pnpm,
                        PackageManager::Yarn,
                    ],
                    fallback: PackageManager::Npm,
                },
            },
            Language::Python => LanguageInfo {
                name: "Python",
                package_managers: SupportedPackageManagers {
                    supported: vec![PackageManager::Poetry, PackageManager::Pip],
                    fallback: PackageManager::Pip,
                },
            },
            Language::Ruby => LanguageInfo {
                name: "Ruby",
                package_managers: SupportedPackageManagers {
                    supported: vec![PackageManager::Bundler],
                    fallback: PackageManager::Bundler,
                },
            },
            Language::Rust => LanguageInfo {
                name: "Rust",
                package_managers: SupportedPackageManagers {
                    supported: vec![PackageManager::Cargo],
                    fallback: PackageManager::Cargo,
                },
            },
        }
    }
}

// TODO: should language have a PackageMangers property
// should provides list of package managers as well as the default?
// pub struct LanguageInfo {
//     pub language: Language,
//     pub package_managers: PackageManagers,
// }
//
// pub struct PackageManagers {
//     pub package_managers: Vec<PackageManager>,
//     pub default_manager: PackageManager
// }

// LanguageDefinition
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LanguageInfo {
    // id
    pub name: &'static str,
    pub package_managers: SupportedPackageManagers,
}

// Random idea for type to encapsulate
// PackageManagement
// PackageManagers
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SupportedPackageManagers {
    supported: Vec<PackageManager>,
    fallback: PackageManager,
}

// impl SupportedPackageManagers {
//
//     pub fn get_fallback(&self) -> Option<PackageManager> {
//         if self.supported.len() == 1 {
//
//         }
//
//         None
//
//     }
//
// }
