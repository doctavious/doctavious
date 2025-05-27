use std::path::PathBuf;

pub struct Rustdocs {
    pub cwd: PathBuf,
    // --out-dir PATH
    pub name: Option<String>

    // --document-private-items
    // document private items
    // --document-hidden-items
    // document items that have doc(hidden)
    // --markdown-playground-url URL
    // URL to send code snippets to
    // --markdown-no-toc
    // don't include table of contents
    // --playground-url URL
    // URL to send code snippets to, may be reset by
    // --markdown-playground-url or
    // `#![doc(html_playground_url=...)]`
}