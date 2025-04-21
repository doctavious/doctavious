use regex::Regex;

pub(crate) fn parse_line(line: &str) -> Option<(String, Vec<String>)> {
    let trimmed_line = line.trim();
    if trimmed_line.is_empty() || trimmed_line.starts_with("#") {
        return None;
    }

    let fields: Vec<String> = line.split_whitespace().map(str::to_string).collect();
    // if fields.len() == 1 {
    //     info!(
    //         "expected at least two fields for rule in {}: {}",
    //         file.to_string_lossy(),
    //         line
    //     );
    //     continue;
    // }

    if let Some((rule_pattern, rest)) = fields.split_first() {
        Some((rule_pattern.to_owned(), rest.to_vec()))
    } else {
        None
    }
}

/// Transform CODEOWNER and CODENOTIFY patterns to regex
/// CODENOTIFY files mimic the structure of CODEOWNER files so this is a shared function.
/// If we wanted to have the rules diverge at any point each crate would have their own parsing function
pub(crate) fn pattern_to_regex(pattern: &str) -> Result<Regex, regex::Error> {
    let mut pattern = pattern.to_string();

    // If the pattern ends with '/', append '**'
    if pattern.ends_with('/') {
        pattern.push_str("**");
    }

    let pattern = regex::escape(&pattern);
    let pattern = pattern
        .replace(r"/\*\*/", r"/([^/]*/)*")
        .replace(r"\*\*/", r"([^/]+/)*")
        .replace(r"/\*\*", r".*")
        .replace(r"\*\*", r".*")
        .replace(r"\*", r"[^/]*");

    // Add regex anchors
    let pattern = format!("^{}$", pattern);

    Ok(Regex::new(&pattern)?)
}
