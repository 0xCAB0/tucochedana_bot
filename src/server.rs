use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use frankenstein::Update;

use crate::update_handler::process_update_task::ProcessUpdateTask;

pub fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello!" }))
        .route("/webhook", post(parse_update))
}

async fn parse_update(update: Json<Update>) -> (StatusCode, Json<()>) {
    log::info!("New update {:?}", update);
    ProcessUpdateTask::new(update.0);
    (StatusCode::OK, Json(()))
}
