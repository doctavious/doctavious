use std::borrow::Cow;
use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};

use crate::framework::{FrameworkDetectionItem, FrameworkDetector, FrameworkMatchingStrategy};
use crate::framework_detection::Detectable;
use crate::projects::project_file::ProjectFile;

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
        Cow::Owned(self.project_files.iter()
            .filter_map(|p| ProjectFile::from_path(p).ok())
            .collect::<Vec<ProjectFile>>())
    }

    fn get_configuration_files(&self) -> &Vec<PathBuf> {
        &self.configs
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
