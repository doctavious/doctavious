// Following from UPM....
// LanguageBackend is the core abstraction of UPM. It represents an
// implementation of all the core package management functionality of
// UPM, for a specific programming language and package manager. For
// example, python-python3-poetry and python-python2-poetry would be
// different backends, as would python-python3-poetry and
// python-python3-pipenv.
//
// Most of the fields of this struct are mandatory, and the Check
// method will panic at UPM startup if they are not provided. Not all
// language backends necessarily need to implement all operations; in
// this case, the relevant functions should call util.NotImplemented,
// which will cause UPM to exit with an appropriate error message.
// (The limitation should be noted in the backend feature matrix in
// the README.)

use std::fs;
use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;
use crate::{Cifrs, CifrsResult};
use crate::package_manager::PackageManager;

use crate::projects::project_file::ProjectFile;

// Module backends contains the language-specific Doctavious Cifrs backends,
// and logic for selecting amongst them. This can generally be thought of as
// languages however "languages" as a concept proves cumbersome given support
// for .NET which is a framework/runtime that supports multiple languages such
// as C# and F#. We could have listed both for a given framework however I
// think they would be handled the same way.
// The term backend was chosen because as framework/runtime are already used
// in our domain and I was sold on the name when I saw it used in Replit's UPM.
mod dotnet;
mod nodejs;
mod python;
mod ruby;
mod rust;

// in UPM they had individual language backends for each supported package manager e.g. Yarn, NPM,
// Poetry, etc. We initially went with having a package_manager module and to create package managers
// for each. I think I prefer having backends be tied to language and then split out based package manager.

#[remain::sorted]
#[non_exhaustive]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageBackends {
    DotNet,
    Go,
    JavaScript,
    Python,
    Ruby,
    Rust,
}

impl LanguageBackends {

    pub fn project_files(&self) -> &[ProjectFile] {
        match self {
            LanguageBackends::DotNet => &[ProjectFile::MsBuild],
            LanguageBackends::Go => &[ProjectFile::GoMod],
            LanguageBackends::JavaScript => &[ProjectFile::PackageJson],
            LanguageBackends::Python => &[
                ProjectFile::PyProject,
                ProjectFile::PipFile,
                ProjectFile::RequirementsTxt,
            ],
            LanguageBackends::Ruby => &[ProjectFile::GemFile],
            LanguageBackends::Rust => &[ProjectFile::CargoToml],
        }
    }

    fn get_project_file(&self) -> Option<PathBuf> {
        for p in self.project_files() {
            for path in p.get_project_paths() {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }
}

pub struct LanguageBackend {
    /// The name of the language backend
    pub name: String,

    // we were calling this a project_file
    // pub spec_file: String,

    pub project_files: Vec<String>

}

// build systems arent necessarily tied to a run time e.g. buck2 can be used across languages

// language runtime
// has project file(s)
// has supported package managers however project file is used to determine which package manager
// is appropriate
//


// vercel has the following logic to detect projects
// from the cwd they call `getWorkspaces` with a max traversal depth of 3
// - calls `detectWorkspaceManagers` which is `detectFramework` but with a list of `workspaceManagers` (yarn, pnpm, rush, nx, etc) and returns first match as workspaceType
// - returns list of type and paths/dirs
// for each workspace they call `getWorkspacePackagePaths`
// - reads corresponding files from workspace paths based on type to get list of package paths
// for each package path they call `detectFrameworks` with a list of possible frameworks
// return map of path and associated frameworks
