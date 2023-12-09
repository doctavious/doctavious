use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::hash::{Hash, Hasher};
use crate::{CasError, CasResult};

// TODO: do we want to include hash algo
#[derive(Deserialize, Serialize)]
pub(crate) struct MerkleTree {
    pub root: TreeNode,
    // May change this to skip serialization and write a custom deserializer to populate index.
    // Will see how large payload is with index and see what the runtime cost is to populate as part
    // of deserialization. We'll also be compressing payload so it might not be to bad
    // Will want test with trees around 50K - 60K
    // this might also be reversed. if multiple paths lead to the same hash we really only need to
    // store the file one. granted this index and how we collect and store are separate concerns
    // like if we add the same file in a separate place we want to insert that record but dont need
    // to store the file just need to store the path and reference.
    pub idx: HashMap<PathBuf, String>,
}

impl MerkleTree {

    // Given we are a CAS I think the following are true
    // we'll store the file using its hash as the file name
    // we dont care about the path for storage (at some point we do want to prune old files)
    // we do care about the path for serving. To serve we need to map path to hash
    // to determine if a file needs to be updated
    // - if path is not present in the current (original) tree
    // - if present in the current (original) but hash is different
    pub fn has_path<P: AsRef<Path>>(&self, path: P) -> bool {
        if let Some(hash) = self.idx.get(path.as_ref()) {
            return Hash::new(path.as_ref().to_string_lossy().as_bytes()).to_string() == *hash;
        }

        false
    }

}


#[derive(Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub(crate) enum MerkleTreeNode {
    Tree(TreeNode),
    Blob(LeafNode),
}

impl MerkleTreeNode {

    pub fn get_hash(&self) -> &str {
        match self {
            MerkleTreeNode::Tree(t) => t.hash.as_str(),
            MerkleTreeNode::Blob(b) => b.hash.as_str()
        }
    }

    pub fn get_path(&self) -> &Path {
        match self {
            MerkleTreeNode::Tree(t) => t.path.as_path(),
            MerkleTreeNode::Blob(b) => b.path.as_path()
        }
    }

}

#[derive(Deserialize, Serialize)]
pub(crate) struct TreeNode {
    hash: String,
    path: PathBuf,
    children: Vec<MerkleTreeNode>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct LeafNode {
    hash: String,
    path: PathBuf,
}

impl MerkleTree {
    pub fn from_path<P: AsRef<Path>>(path: P) -> CasResult<MerkleTree> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Err(CasError::InvalidMerkleTreeEntry(path.to_path_buf()));
        }

        let mut idx: HashMap<PathBuf, String> = HashMap::new();
        let root = MerkleTree::create_tree(path, &mut idx)?;

        Ok(MerkleTree { root, idx })
    }

    fn create_tree<P: AsRef<Path>>(
        path: P,
        idx: &mut HashMap<PathBuf, String>,
    ) -> CasResult<TreeNode> {
        let mut hasher = Hasher::new();
        let mut children = vec![];

        let mut paths = fs::read_dir(&path)?
            .filter_map(Result::ok)
            .collect::<Vec<DirEntry>>();
        paths.sort_by_key(|dir| dir.file_name());

        for entry in paths {
            let path = entry.path();
            if path.is_dir() {
                // If the entry is a directory, recursively create a Merkle tree for it
                let tree = MerkleTree::create_tree(&path, idx)?;
                hasher.update(&tree.hash.to_owned().as_bytes());
                children.push(MerkleTreeNode::Tree(tree));
            } else {
                // If the entry is a file, read its content and hash it
                let path = MerkleTree::normalize(&path);
                let file_hash = MerkleTree::hash_file(&path)?;
                idx.entry(path.to_owned()).or_insert(file_hash.to_owned());

                hasher.update(file_hash.as_bytes());
                children.push(MerkleTreeNode::Blob(LeafNode {
                    hash: file_hash,
                    path,
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
    // dont think latency is not much of a factor, comparatively, as it will be part of a deploy which
    // is generally slower so it might just make more sense to store the tree in some compressed form
    // could just be json compressed (pick a compression algo) and we download it as part of the
    // deployment to do the diff and then upload the new tree. The alternative is storing a tree
    // structure in postgres and having to do a recursive tree search to do the diff. The recursive
    // search might not be bad in general would assume most parts of the tree dont change between
    // deployments
    pub fn diff<'a>(current: &'a MerkleTree, new: &'a MerkleTree) -> Vec<&'a MerkleTreeNode> {
        if new.root.hash == current.root.hash {
            return vec![];
        }

        MerkleTree::diff_tree(current, &new.root)
    }

    fn diff_tree<'a>(current: &'a MerkleTree, new: &'a TreeNode) -> Vec<&'a MerkleTreeNode> {
        let mut diff = vec![];
        for child in &new.children {
            match child {
                MerkleTreeNode::Tree(t) => {
                    if !current.has_path(t.path.as_path()) {
                        let child_diff = MerkleTree::diff_tree(&current, &t);
                        if !child_diff.is_empty() {
                            diff.extend(child_diff);
                        }
                    }
                }
                MerkleTreeNode::Blob(ref b) => {
                    if !current.has_path(b.path.as_path()) {
                        diff.push(&child);
                    }
                }
            }
        }

        diff
    }

    fn hash_file<P: AsRef<Path>>(path: P) -> CasResult<String> {
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
    fn normalize<P: AsRef<Path>>(path: P) -> PathBuf {
        PathBuf::from_str(path.as_ref().to_string_lossy().trim_start_matches("./"))
            .expect("Should be able to normalize path")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::tree::MerkleTree;

    #[test]
    fn verify_baz() {
        let two_hash = blake3::hash(
            fs::read_to_string("tests/fixtures/content/baz/two.txt")
                .unwrap()
                .as_bytes(),
        );
        assert_eq!(
            "dc770fff53f50835f8cc957e01c0d5731d3c2ed544c375493a28c09be5e09763",
            two_hash.to_string()
        );

        let baz_directory_hash = MerkleTree::from_path("tests/fixtures/content/baz")
            .unwrap()
            .root
            .hash;
        assert_eq!(
            baz_directory_hash.to_string(),
            blake3::hash(two_hash.to_string().as_bytes()).to_string()
        );
    }

    #[test]
    fn verify_tree() {
        let baz_hash = MerkleTree::from_path("tests/fixtures/content/baz")
            .unwrap()
            .root
            .hash;
        assert_eq!(
            "f769a64df441b878665034911564c39c8d0ed26df8c1109f0bf4e19edcf51c3c",
            baz_hash.to_string()
        );

        let foo_hash = MerkleTree::from_path("tests/fixtures/content/foo")
            .unwrap()
            .root
            .hash;
        assert_eq!(
            "3f1a91c1d5f0fb6bcf218e9ed33b12332b27b019889fd879618079c08ee20490",
            foo_hash.to_string()
        );

        let one_hash = blake3::hash(
            fs::read_to_string("tests/fixtures/content/one.txt")
                .unwrap()
                .as_bytes(),
        );
        assert_eq!(
            "d33fb48ab5adff269ae172b29a6913ff04f6f266207a7a8e976f2ecd571d4492",
            one_hash.to_string()
        );

        let mut hasher = blake3::Hasher::new();
        hasher.update(baz_hash.to_string().as_bytes());
        hasher.update(foo_hash.to_string().as_bytes());
        hasher.update(one_hash.to_string().as_bytes());
        hasher.update(one_hash.to_string().as_bytes());

        let tree = MerkleTree::from_path("./tests/fixtures/content").unwrap();
        assert_eq!(tree.root.hash, hasher.finalize().to_string());

        println!("{}", serde_json::to_string(&tree).unwrap());
    }
}
