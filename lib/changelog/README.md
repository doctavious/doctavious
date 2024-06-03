# Changelog


## Conventional Commits

https://www.conventionalcommits.org/en/v1.0.0/

## Formats

https://docs.gitlab.com/ee/user/project/changelogs.html

## Test Repos
What are some good repos to test with

- https://github.com/rust-lang/rust-clippy/blob/master/CHANGELOG.md
- https://github.com/tokio-rs/axum/blob/main/axum/CHANGELOG.md
- https://www.cockroachlabs.com/docs/releases/
- https://www.cockroachlabs.com/docs/releases/v23.2
- Poetry


https://cockroachlabs.atlassian.net/wiki/spaces/CRDB/pages/73072807/Git+Commit+Messages
https://cockroachlabs.atlassian.net/wiki/spaces/CRDB/pages/186548364/Release+notes

## TODO

- support multiple changelog formats
- allow to generate changelog for multiple projects within monorepo
- allow to bump specific project within monorepo
- look at git pathspec for include / exclude paths
  - `git log`  ex `git log --oneline -20 -- ':(top)config/**' ':(top)website/**' ':(exclude,top)website/blog/**'`
    - `[--] <path>...`
    - `--grep-reflog=<pattern>`
    - `--grep=<pattern>`
    - `--exclude=<glob-pattern`
- try and support https://github.com/orhun/git-cliff/issues/225