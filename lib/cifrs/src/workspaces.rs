use serde_derive::{Deserialize, Serialize};
use crate::framework::FrameworkDetector;

pub const WORKSPACES_STR: &str = include_str!("workspaces.yaml");

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub project_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub configs: Vec<String>,
    pub detection: FrameworkDetector,
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
