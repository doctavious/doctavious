use serde_derive::Serialize;

use crate::framework::{FrameworkDetectionItem, FrameworkDetector, FrameworkMatchingStrategy};

// TODO: could add PDM and Anaconda (Python)
#[non_exhaustive]
#[derive(Serialize)]
pub enum PackageManager {
    Cargo,
    Go,
    Npm,
    Nuget,
    Poetry,
    Pip,
    Pnpm,
    Bundler,
    Yarn,
}

#[derive(Serialize)]
pub struct PackageManagerInfo {
    pub name: &'static str,
    pub install_command: &'static str,

    // TODO: do we want to change to known_project_files?
    // we would also bring the concept of a exact known file or something like glob
    // for cases in which we dont have a known file ex: dotnet .csproj files
    // pub manifests: &'a [&'static str],
    // pub project_files: &'a [ProjectFile],

    // TODO: multiple files?
    pub lock_file: &'static str,

    // TODO: should we use something specific to package managers?
    // maybe it makes more sense as a trait?
    pub detection: FrameworkDetector,
}

// pub enum ProjectPaths {
//     WellKnown(Vec<&'static str>),
//     Glob(Vec<&'static str>)
// }

impl<'a> PackageManager {
    pub const ALL: &'a [PackageManager] = &[
        PackageManager::Cargo,
        PackageManager::Go,
        PackageManager::Npm,
        PackageManager::Poetry,
        PackageManager::Pip,
        PackageManager::Pnpm,
        PackageManager::Bundler,
        PackageManager::Yarn,
    ];

    pub fn info(&self) -> PackageManagerInfo {
        match self {
            PackageManager::Cargo => PackageManagerInfo {
                name: "cargo",
                install_command: "cargo add",
                lock_file: "Cargo.lock",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::File {
                            path: "Cargo.lock",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "Cargo.toml",
                            content: None,
                        },
                    ],
                },
            },
            PackageManager::Go => {
                PackageManagerInfo {
                    name: "go",
                    install_command: "go get",
                    // TODO: not sure this is appropriate for a lock file
                    lock_file: "go.sum",
                    detection: FrameworkDetector {
                        matching_strategy: FrameworkMatchingStrategy::Any,
                        detectors: vec![FrameworkDetectionItem::File {
                            path: "go.sum",
                            content: None,
                        }],
                    },
                }
            }
            PackageManager::Npm => {
                PackageManagerInfo {
                    name: "npm",
                    install_command: "npm install",
                    lock_file: "package-lock.json",
                    // TODO: if package.json is found with no packageManager should we default
                    // to NPM? Which would mean we would be forced to make sure its at the end
                    // or a way to say content not present
                    detection: FrameworkDetector {
                        matching_strategy: FrameworkMatchingStrategy::Any,
                        detectors: vec![
                            FrameworkDetectionItem::File {
                                path: "package-lock.json",
                                content: None,
                            },
                            FrameworkDetectionItem::File {
                                path: "package.json",
                                content: Some(r#""packageManager":\\s*"npm@.*"""#),
                            },
                        ],
                    },
                }
            }
            PackageManager::Nuget => PackageManagerInfo {
                name: "nuget",
                install_command: "dotnet add",
                lock_file: "packages.lock.json",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![FrameworkDetectionItem::File {
                        path: "packages.lock.json",
                        content: None,
                    }],
                },
            },
            PackageManager::Poetry => PackageManagerInfo {
                name: "poetry",
                install_command: "poetry install",
                lock_file: "poetry.lock",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::File {
                            path: "poetry.lock",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "pyproject.toml",
                            content: Some("[tool.poetry]"),
                        },
                    ],
                },
            },
            PackageManager::Pip => PackageManagerInfo {
                name: "pip",
                install_command: "pip install",
                lock_file: "pipfile.lock",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::File {
                            path: "pipfile.lock",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "pipfile",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "requirements.txt",
                            content: None,
                        },
                    ],
                },
            },
            PackageManager::Pnpm => PackageManagerInfo {
                name: "pnpm",
                install_command: "pnpm install",
                lock_file: "pnpm-lock.yaml",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::File {
                            path: "pnpm-lock.yaml",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "package.json",
                            content: Some(r#""packageManager":\\s*"pnpm@.*""#),
                        },
                    ],
                },
            },
            PackageManager::Bundler => PackageManagerInfo {
                name: "bundler",
                install_command: "bundle install",
                lock_file: "Gemfile.lock",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::File {
                            path: "Gemfile.lock",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "Gemfile",
                            content: None,
                        },
                    ],
                },
            },
            PackageManager::Yarn => PackageManagerInfo {
                name: "yarn",
                install_command: "yarn install",
                lock_file: "yarn.lock",
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::File {
                            path: "yarn.lock",
                            content: None,
                        },
                        FrameworkDetectionItem::File {
                            path: "package.json",
                            content: Some(r#""packageManager":\\s*"yarn@.*""#),
                        },
                    ],
                },
            },
        }
    }

    // pub fn has_dependency(&self, dependency: &str) -> bool {
    //     for p in self.info().project_files {
    //         let found = p.has_dependency(dependency);
    //         // TODO: do we want to log error that file could not be read? Do we want to separate
    //         // file doesnt exist and file cannt be read?
    //         if found.is_ok() {
    //             return true
    //         }
    //     }
    //     false
    //     // match self {
    //     //     PackageManager::NPM => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //
    //     //             if root.get("dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     //     PackageManager::Poetry => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //             if root.get("tool.poetry.dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     //     PackageManager::PIP => {
    //     //         // TODO: handle requirements.txt and Pipfile
    //     //     }
    //     //     PackageManager::PNPM => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //             // might be to do these individual lookup
    //     //             if root.get("tool.poetry.dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     //     PackageManager::RubyGems => {
    //     //         // will a contains be good enough?
    //     //     }
    //     //     PackageManager::Yarn => {
    //     //         for manifest in &self.info().manifests {
    //     //             // TODO: read_config to Value
    //     //             let root: Value = serde_json::from_str(manifest)?;
    //     //             if root.get("dependencies").and_then(dependency).is_some() {
    //     //                 return true
    //     //             }
    //     //         }
    //     //         false
    //     //     }
    //     // }
    // }
}

// impl FromStr for PackageManager {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "cargo" => Ok(PackageManager::Cargo),
//             "pnpm" => Ok(PackageManager::Pnpm),
//             "yarn" => Ok(PackageManager::Yarn),
//             "npm" => Ok(PackageManager::Npm),
//             _ => Err("Invalid package manager".to_string()),
//         }
//     }
// }

// JavaScript
// npm - package-lock.json, package.json
// yarn - yarn.lock, package.json
// pnpm - pnpm-lock.yaml
// pub struct Npm {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Npm {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "npm",
//                 install_command: "npm install",
//                 manifests: vec!["package.json"],
//                 lock_file: "package-lock.json",
//             },
//         }
//     }
// }
//
// pub struct Yarn {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Yarn {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "yarn",
//                 install_command: "yarn install",
//                 manifests: vec!["package.json"],
//                 lock_file: "yarn.lock",
//             },
//         }
//     }
// }
//
// pub struct Pnpm {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Pnpm {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "pnpm",
//                 install_command: "pnpm install",
//                 manifests: vec!["package.json"],
//                 lock_file: "pnpm-lock.yaml",
//             },
//         }
//     }
// }
//
// pub struct Pip {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Pip {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "pip",
//                 install_command: "pip install",
//                 manifests: vec!["pipfile", "requirements.txt"],
//                 lock_file: "pipfile.lock",
//             },
//         }
//     }
// }
//
// pub struct Poetry {
//     pub info: PackageManagerInfo
// }
//
// impl Default for Poetry {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "poetry",
//                 install_command: "poetry install",
//                 manifests: vec!["pyproject.toml"],
//                 lock_file: "poetry.lock",
//             },
//         }
//     }
// }
//
// pub struct RubyGems {
//     pub info: PackageManagerInfo
// }
//
// impl Default for RubyGems {
//     fn default() -> Self {
//         Self {
//             info: PackageManagerInfo {
//                 name: "rubygems",
//                 install_command: "",
//                 manifests: vec!["Gemfile"],
//                 lock_file: "Gemfile.lock",
//             },
//         }
//     }
// }

// JavaScript
// npm - package-lock.json, package.json
// yarn - yarn.lock, package.json
// pnpm - pnpm-lock.yaml

// Python
// pip - requirements.txt, pipfile.lock, pipfile, setup.py
// poetry - poetry.lock, pyproject.toml

// Ruby
// Gems - Gemfile.lock, Gemfile
