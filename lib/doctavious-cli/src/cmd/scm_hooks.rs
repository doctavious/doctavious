pub mod init;
pub mod install;
pub mod list;
pub mod run;
pub mod uninstall;

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// idea from rusty-hook and left-hook
// TODO: flush this out more

// add hook
// execute hook

fn init() {}
