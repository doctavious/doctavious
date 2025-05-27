use std::fs;
use std::path::Path;
use std::process::Command;

// use cyclonedx_bom::models::tool::{Tool, Tools};
// use cyclonedx_bom::prelude::*;

pub struct BillOfMaterials {}

// TODO: this is just a test method to work through some details with cycloneDX.
// will need to build this up once we work with some of the initial kinks
// Ref: https://github.com/CycloneDX/cyclonedx-node-npm/tree/main/src
// Ref: https://cyclonedx.github.io/cdxgen/#/
fn handle_package_json(file: &Path) {
    let package_json = fs::read_to_string(file).unwrap();
    // let bom = Bom {
    //     version: 0,
    //     serial_number: None,
    //     metadata: None,
    //     components: None,
    //     services: None,
    //     external_references: None,
    //     dependencies: None,
    //     compositions: None,
    //     properties: None,
    //     vulnerabilities: None,
    //     signature: None,
    //     annotations: None,
    //     formulation: None,
    //     spec_version: Default::default(),
    // };
}

// -r, --recurse: Recurse mode suitable for mono-repos.
// -o, --output: Output file. Default bom.json
// -t, --type: Project type. Please refer to https://cyclonedx.github.io/cdxgen/#/PROJECT_TYPES
// --deep: Perform deep searches for components. Useful while scanning C/C++ apps, live OS and oci images.
// --project-group: Dependency track project group
// --project-name: Dependency track project name. Default use the directory name
// --project-version: Dependency track project version [string] [default: ""]
// --project-id: Dependency track project id. Either provide the id or the project name and version together
// --parent-project-id: Dependency track parent project id
// --required-only: Include only the packages with required scope on the SBOM
// --server: Run cdxgen as a server
// --server-host: Listen address [default: "127.0.0.1"]
// --server-port: Listen port [default: "9090"]
// --install-deps: Install dependencies automatically for some projects. Defaults to true but disabled for containers and oci scans.
// --evidence: Generate SBOM with evidence for supported languages. [default: false]
// --filter: Filter components containing this word in purl or component.properties.value. Multiple values allowed.
// --only: Include components only containing this word in purl. Useful to generate BOM with first party components alone. Multiple values allowed.
// --author: The person(s) who created the BOM. Set this value if you're intending the modify the BOM and claim authorship. [default: "OWASP Foundation"]
// --profile: BOM profile to use for generation. Default generic. [choices: "appsec", "research", "operational", "threat-modeling", "license-compliance", "generic", "machine-learning", "ml", "deep-learning", "ml-deep", "ml-tiny"] [default: "generic"]
// --exclude: Additional glob pattern(s) to ignore
// --standard: The list of standards which may consist of regulations, industry or organizational-specific standards, maturity models, best practices, or any other requirements which can be evaluated against or attested to. [choices: "asvs-5.0", "asvs-4.0.3", "bsimm-v13", "masvs-2.0.0", "nist_ssdf-1.1", "pcissc-secure-slc-1.1", "scvs-1.0.0", "ssaf-DRAFT-2023-11"]
// --min-confidence: Minimum confidence needed for the identity of a component from 0 - 1, where 1 is 100% confidence. [default: 0]
// --technique: Analysis technique to use [choices: "auto", "source-code-analysis", "binary-analysis", "manifest-analysis", "hash-comparison", "instrumentation", "filename"]

fn scan(cwd: Option<&Path>) {
    let mut command = Command::new("cdxgen");
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    // TODO: do we want to specify language? Maybe that should be optional and we recursively detect by default
    // "-o".to_string()
    let mut args: Vec<String> = vec!["-r".to_string()];

    let output = command.args(args).output().unwrap().stdout;
}

#[cfg(test)]
mod tests {
    use crate::scan;

    #[test]
    fn basic() {
        scan(None);
    }
}
