// implementation base on https://github.com/systeminit/si/blob/main/lib/object-tree/src/lib.rs

//! An tree structure over an arbitrary node type that is cryptographically hashed as a Merkle
//! n-ary acyclic directed graph (i.e. a Merkle DAG).
//!
//! # Supporting Reference Literature

// Mention Git CAS

// this is similar to what Netlify does as part of their deploy see
// https://github.com/netlify/cli/blob/main/src/utils/deploy/hash-files.mjs#L10.
// I'm keeping it for now
// although I question whether it would be better to just create an actual merkle tree to use to do
// the diff. I dont know the Netlify server implementation but just sending map of path and hash
// seems like it would be inefficient as you would have to look at each file whereas I would think
// with a merkle tree you could check tree hashes and skip sections if they matched?

// vercel implementation

use std::path::PathBuf;

use thiserror::Error;

mod hash;
mod tree;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CasError {
    /// Error that occurs when an invalid path is passed to when constructing Merkle Tree
    #[error("Invalid Merkle Tree entry point. {0} must be a directory")]
    InvalidMerkleTreeEntry(PathBuf),

    /// Error that may occur while I/O operations.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type CasResult<T> = Result<T, CasError>;
