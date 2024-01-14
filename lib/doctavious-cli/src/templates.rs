use std::fs;
use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::markup_format::MarkupFormat;
use crate::templating::TemplateType;


pub(crate) fn get_template(dir: &Path, template_type: TemplateType, extension: &str) -> PathBuf {
    // see if direction defines a custom template
    let custom_template = dir
        .join(template_type.get_default_path())
        .with_extension(extension);

    if custom_template.is_file() {
        custom_template
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(template_type.get_default_path().with_extension(extension))
    }
}

pub(crate) fn get_template_content(
    dir: &Path,
    template_type: TemplateType,
    extension: &str,
) -> String {
    let template_path = get_template(dir, template_type, extension);
    // TODO: we shouldnt panic here
    fs::read_to_string(&template_path).expect(&format!(
        "failed to read file {}.",
        &template_path.to_string_lossy()
    ))
}

pub(crate) fn get_title<R>(rdr: R, markup_format: MarkupFormat) -> String
where
    R: BufRead,
{
    // TODO: swap this implementation for AST when ready
    let leading_char = markup_format.leading_header_character();
    for line in rdr.lines() {
        let line = line.unwrap();
        if line.starts_with(leading_char) {
            let last_hash = line
                .char_indices()
                .skip_while(|&(_, c)| c == leading_char)
                .next()
                .map_or(0, |(idx, _)| idx);

            // Trim the leading hashes and any whitespace
            return line[last_hash..].trim().to_string();
        }
    }

    String::new()
}
