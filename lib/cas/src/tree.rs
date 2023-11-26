// implementation base on https://github.com/systeminit/si/blob/main/lib/object-tree/src/lib.rs

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufReader, Read};
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::{Hash, Hasher};

// TODO: do we want to include hash algo
#[derive(Deserialize, Serialize)]
pub(crate) struct MerkleTree {
    pub root: TreeNode,
    // May change this to skip serialization and write a custom deserializer to populate index.
    // Will see how large payload is with index and see what the runtime cost is to populate as part
    // of deserialization. We'll also be compressing payload so it might not be to bad
    // Will want test with trees around 50K - 60K
    pub idx: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum MerkleTreeNode {
    Tree(TreeNode),
    Blob(LeafNode),
}

#[derive(Deserialize, Serialize)]
pub(crate) struct TreeNode {
    hash: String,
    path: String,
    children: Vec<MerkleTreeNode>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct LeafNode {
    hash: String,
    // should this be path?
    file_name: String,
}

impl MerkleTree {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<MerkleTree, Box<dyn Error>> {
        let path = path.as_ref();
        path.try_exists()?;
        if path.is_file() {
            // TODO: return error that path must be a directory
        }

        // map hash to file paths
        // hash: [file, file, ...]
        let mut idx: HashMap<String, Vec<String>> = HashMap::new();
        // normalizedPath: hash
        // let mut files: HashMap<String, String> = HashMap::new();
        let root = MerkleTree::create_tree(path, &mut idx)?;

        Ok(MerkleTree { root, idx })
    }

    fn create_tree<P: AsRef<Path>>(
        path: P,
        idx: &mut HashMap<String, Vec<String>>,
    ) -> Result<TreeNode, Box<dyn Error>> {
        let mut hasher = Hasher::new();
        let mut children = vec![];
        // for entry in WalkDir::new(&path).contents_first(true) {

        let mut paths = fs::read_dir(&path)?
            .filter_map(Result::ok)
            .collect::<Vec<DirEntry>>();
        paths.sort_by_key(|dir| dir.path());

        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // If the entry is a directory, recursively create a Merkle tree for it
                let tree = MerkleTree::create_tree(&path, idx)?;
                hasher.update(&tree.hash.to_owned().as_bytes());
                children.push(MerkleTreeNode::Tree(tree));
            } else {
                // If the entry is a file, read its content and hash it
                // let path = path.normalize()?.as_path().to_string_lossy().to_string();
                let path = MerkleTree::normalize(&path);
                let file_hash = MerkleTree::hash_file(&path)?;
                idx.entry(file_hash.to_owned())
                    .or_default()
                    .push(path.to_owned());

                hasher.update(file_hash.as_bytes());
                children.push(MerkleTreeNode::Blob(LeafNode {
                    hash: file_hash,
                    file_name: path.to_owned(),
                }));
            }
        }

        let directory_hash = hasher.finalize();
        return Ok(TreeNode {
            hash: directory_hash.to_string(),
            path: MerkleTree::normalize(path),
            children,
        });
    }

    // what should this return? Vec of subtrees that arent in the original?
    // what about list of updated files hashes? this is probably the second thing that needs to occur
    // For each subtree that is different we would need to store it and all children for the tree
    // but we wouldnt necessarily need to upload the files.
    // Noting the though here for now...trees generally wont change very often and when they do I
    // dont think latency is much of a factor, comparatively, as it will be part of a deploy which
    // is generally slower so it might just make more sense to store the tree in some compressed form
    // could just be json compressed (pick a compression algo) and we download it as part of the
    // deployment to do the diff and then upload the new tree. The alternative is storing a tree
    // structure in postgres and having to do a recursive tree search to do the diff. The recursive
    // search might not be bad in general would assume most parts of the tree dont change between
    // deployments
    pub fn diff(original: MerkleTree, updated: MerkleTree) {}

    fn hash_file<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
        let input = File::open(path)?;
        let mut reader = BufReader::new(input);

        let mut hasher = Hasher::new();
        let mut buffer = [0; 1024];

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        let file_hash = hasher.finalize();
        Ok(file_hash.to_string())
    }

    fn hash_directory(children: Vec<MerkleTreeNode>) -> Hash {
        let mut hasher = Hasher::new();
        for child in children {
            let hash = match child {
                MerkleTreeNode::Tree(t) => t.hash,
                MerkleTreeNode::Blob(b) => b.hash,
            };
            hasher.update(hash.as_bytes());
        }
        hasher.finalize()
    }

    // fs::canonicalize results in an absolute path which we dont want so here is a very basic
    // implementation to make sure we adjust path for platform could move this out to a
    // doctavious common/core library
    fn normalize<P: AsRef<Path>>(path: P) -> String {
        std::env::split_paths(path.as_ref())
            .map(|x| {
                x.as_path()
                    .to_string_lossy()
                    .trim_start_matches("./")
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(std::path::MAIN_SEPARATOR_STR)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;

    use crate::tree::MerkleTree;

    #[test]
    fn verify_tree_two() -> Result<(), Box<dyn Error>> {
        let tree = MerkleTree::from_path("tests/fixtures/content").unwrap();

        let two_hash =
            blake3::hash(fs::read_to_string("tests/fixtures/content/baz/two.txt")?.as_bytes());
        let one_hash =
            blake3::hash(fs::read_to_string("tests/fixtures/content/one.txt")?.as_bytes());
        let a_hash =
            blake3::hash(fs::read_to_string("tests/fixtures/content/foo/a.txt")?.as_bytes());
        let b_hash =
            blake3::hash(fs::read_to_string("tests/fixtures/content/foo/b.txt")?.as_bytes());
        let ab_hash =
            blake3::hash(format!("{}{}", a_hash.to_string(), b_hash.to_string()).as_bytes());

        println!("{}", serde_json::to_string(&tree).unwrap());

        Ok(())
    }
}
