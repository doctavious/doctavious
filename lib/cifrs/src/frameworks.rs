use serde_derive::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "slug")]
enum Frameworks {
    #[serde(rename = "antora")]
    Antora(FrameworkInfo),
    #[serde(rename = "astro")]
    Astro(FrameworkInfo),
    #[serde(rename = "docfx")]
    DocFx(FrameworkInfo),
    #[serde(rename = "docusaurus-v2")]
    DocusaurusV2(FrameworkInfo),
    #[serde(rename = "eleventy")]
    Eleventy(FrameworkInfo),
    #[serde(rename = "gatsby")]
    Gatsby(FrameworkInfo),
    #[serde(rename = "Hexo")]
    Hexo(FrameworkInfo),
    #[serde(rename = "Hugo")]
    Hugo(FrameworkInfo),
    #[serde(rename = "Jekyll")]
    Jekyll(FrameworkInfo),
    #[serde(rename = "mdbook")]
    MdBook(FrameworkInfo),
    #[serde(rename = "mkdocs")]
    MkDocs(FrameworkInfo),
    #[serde(rename = "nextjs")]
    Nextjs(FrameworkInfo),
    #[serde(rename = "nextra")]
    Nextra(FrameworkInfo),
    #[serde(rename = "nuxtjs")]
    Nuxtjs(FrameworkInfo),
    #[serde(rename = "sphinx")]
    Sphinx(FrameworkInfo),
    #[serde(rename = "sveltekit")]
    SvelteKit(FrameworkInfo),
    #[serde(rename = "vitepress")]
    Vitepress(FrameworkInfo),
    #[serde(rename = "vuepress")]
    Vuepress(FrameworkInfo),
}

// impl Frameworks {
//
//     fn get_output_dir(&self) -> () {
//         match self {
//             Frameworks::Antora(a) => {
//                 a.id;
//             }
//             Frameworks::Astro(_) => {}
//             Frameworks::DocFx(_) => {}
//             Frameworks::DocusaurusV2(_) => {}
//             Frameworks::Eleventy(_) => {}
//             Frameworks::Gatsby(_) => {}
//             Frameworks::Hexo(_) => {}
//             Frameworks::Hugo(_) => {}
//             Frameworks::Jekyll(_) => {}
//             Frameworks::MdBook(_) => {}
//             Frameworks::MkDocs(_) => {}
//             Frameworks::Nextjs(_) => {}
//             Frameworks::Nextra(_) => {}
//             Frameworks::Nuxtjs(_) => {}
//             Frameworks::Sphinx(_) => {}
//             Frameworks::SvelteKit(_) => {}
//             Frameworks::Vitepress(_) => {}
//             Frameworks::Vuepress(_) => {}
//         }
//     }
//
// }

#[cfg(test)]
mod tests {
    use serde_derive::{Deserialize, Serialize};

    use crate::framework::FrameworkInfo;
    use crate::frameworks::{Frameworks, FRAMEWORKS_STR};

    #[derive(Debug, Deserialize, Serialize)]
    struct SupportedFrameworks {
        // pub frameworks: Vec<Frameworks>,
        pub frameworks: Vec<FrameworkInfo>
    }

    #[test]
    fn test_deserialize_frameworks_yaml() {
        let frameworks: SupportedFrameworks = serde_yaml::from_str(FRAMEWORKS_STR).expect("");
        println!("{}", serde_json::to_string(&frameworks).unwrap());
    }
}
