use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Router,
    routing::{get, post},
};

// TODO: we'll compute the equivalent of the README which will have categories
// then each category with the associated links to the content
// not sure what the path to the "REAME" should be. 
// we'll want to allow search for categories across users.
// we'll also want a generic search on content across users
// Do we want to allow users to link git repo? Seems like this align with our approach for docs
// An alternative is to allow for users to get all files in a flat structure then tag via
// frontmatter what categories content should be part of which we can then create appropriate
// "README"
pub fn get_routes() -> Router {
    Router::new()
        .route("/tils", post(new_til))
        // maybe we'll just group search together under one endpoint
        .route("/tils/:category", get(get_tils_by_category))
        .route("/{user_name}/tils", get(get_tils))
        .route("/{user_name}/tils/{path}", get(get_til))
}

async fn new_til() {

}

async fn get_tils_by_category(
    Path(category): Path<u64>
) {

}

async fn get_tils(
    Path(user_name): Path<u64>,
    Path(id): Path<u64>
) {

}

async fn get_til(
    Path(user_name): Path<u64>,
    Path(id): Path<u64>
) {

}
