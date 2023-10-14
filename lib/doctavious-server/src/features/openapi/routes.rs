use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Router,
    routing::{get, post},
};

pub fn get_routes() -> Router {
    Router::new()
        .route("/projects/:project_id/openapi", get(get_openapi).post(new_openapi))
}

async fn get_openapi(
    Path(project_id): Path<u64>
) {

}

// accept json openapi schema
// validate schema
// store
async fn new_openapi(
    Path(project_id): Path<u64>
) {

}
