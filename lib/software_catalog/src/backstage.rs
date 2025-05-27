use std::collections::HashMap;

// https://backstage.io/docs/features/software-catalog/descriptor-format

pub enum Kind {
    Api,
    Component,
    Domain,
    Group,
    Location,
    Resource,
    System,
    Template,
    User,
}

pub struct Link {
    pub url: String,
    pub title: String,
    pub icon: String,
    pub link_type: String,
}

pub struct Metadata {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub uid: Option<String>, // generated
    pub namespace: Option<String>,
    pub labels: HashMap<String, String>,      // optional
    pub annotations: HashMap<String, String>, // optional
    pub tags: Vec<String>,                    // optional
}

pub struct ComponentEntity {
    pub api_version: String,
    pub kind: Kind,
}
