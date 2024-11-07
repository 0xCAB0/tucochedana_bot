use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use frankenstein::Update;

use crate::{update_handler::process_update::UpdateProcessor, BotError};

pub fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello!" }))
        .route("/webhook", post(parse_update))
}
async fn parse_update(Json(update): Json<Update>) -> axum::response::Result<()> {
    log::info!("New update -> {:#?}", update);
    UpdateProcessor::run(&update).await?;
    Ok(())
}

impl IntoResponse for BotError {
    fn into_response(self) -> axum::response::Response {
        // Customize error response here
        (StatusCode::BAD_REQUEST, "Error processing update").into_response()
    }
}
