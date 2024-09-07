use regex::{Error, Regex, RegexBuilder};

pub fn convert_to_regex(patterns: Option<&Vec<String>>) -> Result<Option<Vec<Regex>>, Error> {
    Ok(if let Some(patterns) = patterns {
        let mut regexs = Vec::new();
        for p in patterns {
            regexs.push(RegexBuilder::new(p).size_limit(100).build()?)
        }
        Some(regexs)
    } else {
        None
    })
}
