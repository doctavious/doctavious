use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use mime;
use opendal::Operator;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// use futures_util::stream::StreamExt;

// Netlify: preparing as the upload manifest is generated, and either prepared, uploading, uploaded, or ready
// Vercel: BUILDING, ERROR, INITIALIZING, QUEUED, READY, CANCELED
pub enum DeploymentStatus {
    Pending,
    Complete,
    Failed,
}

#[derive(Serialize)]
pub struct Deployment {}

pub fn get_routes<S>(state: Operator) -> Router<S> {
    Router::new()
        .route("/deployments", get(get_deployments).post(new_deployment))
        .route("/deployments/{id}", get(get_deployment))
        .route("/deployments/{id}/rollback", post(rollback_deployment))
        .route("/deployments/{id}/files", get(get_files).post(upload_files))
        // I dont love this route but good enough for now
        .route(
            "/deployment/{id}/files/:hash_or_path",
            get(get_file_by_identifier),
        )
        .with_state(state)
}

async fn get_deployments(// TODO: query params
) -> Result<Json<Vec<Deployment>>, StatusCode> {
    Ok(Json(vec![]))
}

// if zip is sent assume its the full content of tree. if its not return error
// Content-Type: application/zip
async fn new_deployment(
    State(operator): State<Operator>,
    mut multipart: Multipart, // json merkle and potentially zip
) {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap().to_string();
        let file_name = &field.file_name().unwrap().to_lowercase();
        let data = field.bytes().await.unwrap();

        println!(
            "Length of `{} / {}` is {} bytes",
            name,
            file_name,
            data.len()
        );
        println!("{}", std::str::from_utf8(&data).unwrap());

        // TODO: dont unwrap
        operator.write(file_name, data).await.unwrap();

        // let mut lister = operator.scan("").await.unwrap();
        // let page = lister.next_page().await.unwrap().unwrap_or_default();
        // for i in page {
        //     print!("{}", i.path());
        // }
    }

    // upload merkle tree file to storage
    // path will be /<tenant_id>/<project_id>/deployments/<deployment_id> ?
    // what is the event(s)? DeploymentCreated? DiffFinished?

    // set storage path (url) in event

    // check for current deployment
    // if first deployment track that all files are required
    // otherwise diff to determine required files

    // store event(s)
}

async fn upload_files(
    Path(id): Path<u64>,
    State(operator): State<Operator>,
    mut multipart: Multipart,
) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }
}

// TODO: have a API to get all files for the current deployment
#[derive(Serialize)]
pub struct DeploymentFile {
    id: String,
    path: String,
    hash: String,
    mime_type: String,
    size: u64,
}

// TODO: You can get the raw contents of the file by using the custom media type application/vnd.bitballoon.v1.raw as the Content-Type of your HTTP request.

// Vercel has...
// /v6/deployments/{id}/files
// Allows to retrieve the file structure of a deployment by supplying the deployment unique identifier.

// We want to allow users to get raw content as well as the metadata
// we also want to provide file structure of a deployment which I think
// for us is to just return the merkle tree.
// We also will need a way for the user to download the docs which would mean
// creating a zip file based on the merkle tree. if we build it once we can keep it
// around until we delete the deployment. I suppose we could pre-emptively do this during
// the initial deployment but I feel like users typically wont do this (they have the files in git)
// and thus no need to pay for the storage

// Vercel has
/** A deployment file tree entry */
// interface FileTree {
//     /** The name of the file tree entry */
//     name: string
//     /** String indicating the type of file tree entry. */
//     type: "directory" | "file" | "symlink" | "lambda" | "middleware" | "invalid"
//     /** The unique identifier of the file (only valid for the `file` type) */
//     uid?: string
//     /** The list of children files of the directory (only valid for the `directory` type) */
//     children?: __REF__FileTree__[]
//     /** The content-type of the file (only valid for the `file` type) */
//     contentType?: string
//     /** The file "mode" indicating file type and permissions. */
//     mode: number
//     /** Not currently used. See `file-list-to-tree.ts`. */
//     symlink?: string
// }

// /** This object contains information related to the pagination of the current request, including the necessary parameters to get the next or previous page of data. */
// interface Pagination {
//     /** Amount of items in the current page. */
//     count: number
//     /** Timestamp that must be used to request the next page. */
//     next: number | null
//     /** Timestamp that must be used to request the previous page. */
//     prev: number | null
// }

// error-response
// All API endpoints contain a code and message within the error responses, though some API endpoints extend the error object to contain other information. Each endpoint that does this will be documented in their appropriate section.
// While we recommend that you write error messages that fit your needs and provide your users with the best experience, our message fields are designed to be neutral, not contain sensitive information, and can be safely passed down to user interfaces
// {
//     "error": {
//       "code": "forbidden",
//       "message": "Not authorized"
//     }
// }

// rate limits
// X-RateLimit-Limit
// The maximum number of requests that the consumer is permitted to make.
// X-RateLimit-Remaining
// The number of requests remaining in the current rate limit window.
// X-RateLimit-Reset
// The time at which the current rate limit window resets in UTC epoch seconds.

// TODO: does this need to return Json<User>
/// Geta list of of all files for the deployment
async fn get_files(Path(id): Path<u64>) -> Result<Json<Vec<DeploymentFile>>, StatusCode> {
    Ok(Json(vec![DeploymentFile {
        id: String::from("2"),
        path: String::from("/index.html"),
        hash: String::from("hash"),
        mime_type: mime::TEXT_PLAIN.to_string(),
        size: 1,
    }]))
}

// do we want a different route to get raw content or use a header to configure
// what content would we use for header?
// text/plain; charset=utf-8
// application/vnd.bitballoon.v1.raw
// Content-type: application/vnd+company.category+xml
// application/vnd.github.raw
// application/vnd.doctavious.raw
async fn get_file_by_identifier(
    Path(id): Path<u64>,
    Path(hash_or_path): Path<String>,
) -> Result<String, StatusCode> {
    let id = hash_or_path.trim();
    if id.is_empty() {
        // return error
    }

    // TODO: should we enforce paths must start with /?
    if id.contains('.') || id.contains('/') {
        // get file by path
    }

    // get file by hash
    Ok("hello".to_string())
}

fn get_file_by_hash(hash: &str) {}

fn get_file_by_path(path: &str) {}

// this should return details of the deployment including merkle tree
async fn get_deployment(Path(id): Path<u64>) -> Result<Json<Deployment>, StatusCode> {
    todo!()
}

async fn rollback_deployment(Path(id): Path<u64>) {}
