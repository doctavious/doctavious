use std::env;
use std::path::PathBuf;
use cifrs::Cifrs;
use crate::CliResult;

// TODO: do we want to offer preview deployments?


pub fn invoke(dir: Option<PathBuf>, build: bool) -> CliResult<()> {
    let cwd = dir.unwrap_or(env::current_dir()?);

    if build {
        // TODO: this should probably return output directory
        Cifrs::build(&cwd, true)?;
    }

    // TODO: generate merkle tree

    // TODO: see if project is linked and if not setup
    // example of how vercel does setup/linking as part of deploy
    // https://github.com/vercel/vercel/blob/cfc1c9e818ebb55d440479cf0edf18536b772b28/packages/cli/src/commands/deploy/index.ts#L274

    // TODO: diff local merkle tree with current deployed to determine what needs to be pushed
    // walk merkle tree and see what portions are new/updated and upload to Doctavious.
    // this of course will require a Doctavious client as well as a doctavious backend.
    // still need to figure out how to store merkle tree.
    // netlify has a great blog post but no details on format of mongodb collections/data

    Ok(())
}