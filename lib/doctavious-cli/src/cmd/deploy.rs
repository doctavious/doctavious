use std::env;
use std::path::PathBuf;

use cifrs::{BuildOutput, Cifrs};

use crate::CliResult;

// TODO: do we want to offer preview deployments?
// TODO: what setup / linking needs to occur? How to make initial onboarding as easy as possible?

pub fn invoke(dir: Option<PathBuf>, prebuilt: bool) -> CliResult<Option<String>> {
    let cwd = dir.unwrap_or(env::current_dir()?);

    // TODO: can only deploy directory
    if cwd.is_file() {}

    // TODO: retrieve project / org from .doctavious
    // get linked project
    // if link status is error/fail return error
    // if not linked see if auto confirm is enabled
    // if not setup return error
    // get org to determine scope to deploy to
    // get project name
    // link folder to project

    // if linked org should be present. if its not error

    // create deploy
    // if deploy is missing project settings - edit project settings then create deploy with
    // updating settings

    let build_dir = if !prebuilt {
        let build_output = Cifrs::build(&cwd, false, true)?;
        match build_output {
            BuildOutput::DryRun => unreachable!(),
            BuildOutput::Invoked(result) => result.dir,
        }
    } else {
        cwd
    };

    let tree = cas::tree::MerkleTree::from_path(build_dir)?;
    println!("{}", serde_json::to_string(&tree).unwrap());

    // TODO: see if project is linked and if not setup
    // example of how vercel does setup/linking as part of deploy
    // https://github.com/vercel/vercel/blob/cfc1c9e818ebb55d440479cf0edf18536b772b28/packages/cli/src/commands/deploy/index.ts#L274

    // TODO: diff local merkle tree with current deployed to determine what needs to be pushed
    // walk merkle tree and see what portions are new/updated and upload to Doctavious.
    // this of course will require a Doctavious client as well as a doctavious backend.
    // still need to figure out how to store merkle tree.
    // netlify has a great blog post but no details on format of mongodb collections/data

    // Ok(CommandOutput::default())

    Ok(None)
}
