use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

// TODO: these probably need to be under a project. can still keep them separate
pub fn get_routes() -> Router {
    Router::new()
        .route("/projects/:project_id/adrs", get(get_adrs).post(new_adr))
        .route("/projects/:project_id/adrs/{id}", get(get_adr))
}

async fn new_adr() {}

async fn get_adrs(Path(project_id): Path<u64>) {}

async fn get_adr(Path(project_id): Path<u64>, Path(id): Path<u64>) {}
