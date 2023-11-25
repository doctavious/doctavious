use lazy_static::lazy_static;

use crate::framework::FrameworkInfo;

pub mod antora;
pub mod astro;
pub mod docfx;
pub mod docusaurus_v2;
pub mod eleventy;
pub mod gatsby;
pub mod hexo;
pub mod hugo;
pub mod jekyll;
pub mod mdbook;
pub mod mkdocs;
pub mod nextjs;
pub mod nextra;
pub mod nuxt_v3;
pub mod nuxtjs;
pub mod sphinx;
pub mod sveltekit;
pub mod vitepress;
pub mod vuepress;

pub const FRAMEWORKS_STR: &str = include_str!("frameworks.yaml");

lazy_static! {

    // TODO: probably doesnt need to be an owned type
    static ref FRAMEWORKS_LIST: Vec<FrameworkInfo> = serde_yaml::from_str::<Vec<FrameworkInfo>>(FRAMEWORKS_STR)
        .expect("frameworks.yaml should be deserializable");
}

pub fn get_all() -> Vec<FrameworkInfo> {
    FRAMEWORKS_LIST.to_vec()
}

#[cfg(test)]
mod tests {
    use crate::frameworks::get_all;

    #[test]
    fn test_deserialize_frameworks_yaml() {
        println!("{}", serde_json::to_string(&get_all()).unwrap());
    }
}
