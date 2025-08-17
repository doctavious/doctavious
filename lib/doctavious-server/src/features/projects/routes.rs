use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

pub fn get_routes() -> Router {
    Router::new()
        .route("/projects", get(get_projects).post(new_project))
        .route("/projects/{id}", get(get_project))
        .route("/projects/{id}/docs", get(get_project_docs))
}

async fn new_project() {}

async fn get_projects() {}

async fn get_project(Path(id): Path<u64>) {}

async fn get_project_docs(Path(id): Path<u64>) {}
