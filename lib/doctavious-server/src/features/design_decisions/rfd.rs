use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Router,
    routing::{get, post},
};

// TODO: these probably need to be under a project. can still keep them separate 
pub fn get_routes() -> Router {
    Router::new()
        .route("/projects/:project_id/rfds", get(get_rfds).post(new_rfd))
        .route("/projects/:project_id/rfds/:id", get(get_rfd))
}

async fn new_rfd() {

}

async fn get_rfds(
    Path(project_id): Path<u64>
) {

}

async fn get_rfd(
    Path(project_id): Path<u64>,
    Path(id): Path<u64>
) {

}

