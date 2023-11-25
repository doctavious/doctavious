use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

use crate::framework::{FrameworkDetectionItem, FrameworkDetector, FrameworkMatchingStrategy};
use crate::framework_detection::Detectable;
use crate::projects::project_file::ProjectFile;

pub const PACKAGE_MANAGER_STR: &str = include_str!("package_managers.yaml");

// pub enum ProjectPaths {
//     WellKnown(Vec<&'static str>),
//     Glob(Vec<&'static str>)
// }

// // TODO: could add PDM and Anaconda (Python)
// #[non_exhaustive]
// #[remain::sorted]
// #[derive(Debug, Deserialize, PartialEq, Serialize)]
// #[non_exhaustive]
// #[serde(rename_all = "lowercase")]
// // #[serde(tag = "id")]
// pub enum PackageManager {
//     Bundler,
//     Cargo,
//     GoModules,
//     Npm,
//     Nuget,
//     Pip,
//     Pnpm,
//     Poetry,
//     Yarn,
// }

lazy_static! {

    // TODO: probably doesnt need to be an owned type
    static ref PACKAGE_MANAGER_MAP: HashMap<String, PackageManagerInfo> = serde_yaml::from_str::<Vec<PackageManagerInfo>>(PACKAGE_MANAGER_STR)
        .expect("package_managers.yaml should be deserializable")
        .iter().map(|v| (v.id.to_string(), v.to_owned()))
        .collect();
}

// TODO: probably doesnt need to be an owned type
pub fn get_list() -> Vec<PackageManagerInfo> {
    PACKAGE_MANAGER_MAP.values().map(|v| v.to_owned()).collect()
}

impl Detectable for PackageManagerInfo {
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

impl Detectable for &PackageManagerInfo {
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PackageManagerInfo {
    pub id: String,
    // pub name: &'static str,
    pub name: String,
    // pub install_command: &'static str,
    pub install_command: String,

    // TODO: do we want to change to known_project_files?
    // we would also bring the concept of a exact known file or something like glob
    // for cases in which we dont have a known file ex: dotnet .csproj files
    // pub manifests: &'a [&'static str],
    // pub project_files: &'a [ProjectFile],

    // TODO: multiple files?
    // pub lock_file: &'static str,
    pub lock_file: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub project_files: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configs: Vec<PathBuf>,

    // TODO: should we use something specific to package managers?
    // maybe it makes more sense as a trait?
    pub detection: FrameworkDetector,
}

impl PackageManagerInfo {
    pub fn find_by_id(id: &str) -> Option<PackageManagerInfo> {
        PACKAGE_MANAGER_MAP.get(id).cloned()
    }

    pub fn find_by_ids(ids: Vec<&str>) -> Vec<PackageManagerInfo> {
        ids.iter()
            .filter_map(|id| PackageManagerInfo::find_by_id(id))
            .collect()
    }
}
