use std::path::PathBuf;

use crate::settings::{
    DEFAULT_ADR_INIT_TEMPLATE_PATH, DEFAULT_ADR_RECORD_TEMPLATE_PATH,
    DEFAULT_ADR_TOC_TEMPLATE_PATH, DEFAULT_RFD_RECORD_TEMPLATE_PATH, DEFAULT_RFD_TOC_TEMPLATE_PATH,
    DEFAULT_TIL_POST_TEMPLATE_PATH, DEFAULT_TIL_TOC_TEMPLATE_PATH,
};

pub enum TemplateType {
    Adr(AdrTemplateType),
    Rfd(RfdTemplateType),
    Til(TilTemplateType),
}
#[derive(Default)]
pub enum AdrTemplateType {
    Init,
    #[default]
    Record,
    ToC,
}

#[derive(Default)]
pub enum RfdTemplateType {
    #[default]
    Record,
    ToC,
}

pub enum TilTemplateType {
    ReadMe,
    Post,
}

impl TemplateType {
    // TODO: should probably consolidate this with other ways to get template paths
    pub fn get_default_path(&self) -> PathBuf {
        let s = match self {
            TemplateType::Adr(templates) => match templates {
                AdrTemplateType::Init => DEFAULT_ADR_INIT_TEMPLATE_PATH,
                AdrTemplateType::Record => DEFAULT_ADR_RECORD_TEMPLATE_PATH,
                AdrTemplateType::ToC => DEFAULT_ADR_TOC_TEMPLATE_PATH,
            },
            TemplateType::Rfd(templates) => match templates {
                RfdTemplateType::Record => DEFAULT_RFD_RECORD_TEMPLATE_PATH,
                RfdTemplateType::ToC => DEFAULT_RFD_TOC_TEMPLATE_PATH,
            },
            TemplateType::Til(templates) => match templates {
                TilTemplateType::ReadMe => DEFAULT_TIL_TOC_TEMPLATE_PATH,
                TilTemplateType::Post => DEFAULT_TIL_POST_TEMPLATE_PATH,
            },
        };

        PathBuf::from(s)
    }
}

// TODO: tests
#[cfg(test)]
mod tests {

    // TODO: invalid template should return valid error
}
