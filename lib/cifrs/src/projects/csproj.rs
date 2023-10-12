use serde_derive::{Deserialize};

/// Required root element of an MSBuild project file.
/// represent a C# project file that contains the list of files included in a project along with
/// the references to system assemblies
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "Project")]
pub(crate) struct CSProj {
    #[serde(rename = "Sdk")]
    pub sdk: String,

    #[serde(rename = "ItemGroup")]
    pub item_groups: Vec<ItemGroup>
}

impl CSProj {

    // if we want to get version this could be get_package_reference and have it return Option
    pub(crate) fn has_package_reference(&self, package_reference: &str) -> bool {
        for item_group in &self.item_groups {
            // could also do item_group.package_references.unwrap_or_default()
            if let Some(package_references ) = &item_group.package_references {
                for pkref in package_references {
                    if package_reference == pkref.include {
                        return true;
                    }
                }
            }
        }

        false
    }

}

/// Contains a set of user-defined Item elements.
/// Every item used in an MSBuild project must be specified as a child of an ItemGroup element.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct ItemGroup {
    #[serde(rename = "PackageReference")]
    pub package_references: Option<Vec<PackageReference>>
}

/// Package references, using <PackageReference> MSBuild items, specify NuGet package dependencies
/// directly within project files
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct PackageReference {
    pub include: String,
    pub version: Option<String>,
}
