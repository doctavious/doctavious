use regex::Regex;

/// Parse CODEOWNER and CODENOTIFY files
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
