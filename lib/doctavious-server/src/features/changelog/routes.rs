use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Router,
    routing::{get, post},
};

// TODO: not sure if we'll keep these specific routes but a placeholder
// We might want to provoide options for changelog format
// 1. one file
// 2. multiple files
// be able to search / filter by labels?

// https://git-scm.com/docs/git-interpret-trailers
// support conventional commits out of the box
// do we want to support grabbing CHANGELOG.md?

// TODO: but this in cli to support changelog:
// https://gitlab.com/gitlab-org/gitlab-foss/-/blob/master/doc/development/changelog.md

// would potentially be nice to output something like https://github.blog/changelog/
// I also like https://www.cockroachlabs.com/docs/releases/v23.1.html
pub fn get_routes() -> Router {
    Router::new()
        .route("/projects/:project_id/changelog", post(new_changelog))
        .route("/projects/:project_id/changelog/:id", get(get_changelog))
}

async fn new_changelog(
    Path(project_id): Path<u64>
) {

}

async fn get_changelog(
    Path(project_id): Path<u64>,
    Path(id): Path<u64>
) {

}
