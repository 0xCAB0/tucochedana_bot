use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use fang::{AsyncQueue, NoTls};
use frankenstein::Update;
use tokio::sync::Mutex;

use crate::{update_handler::process_update::UpdateProcessor, BotError};

pub fn app(queue: AsyncQueue<NoTls>) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello!" }))
        .route("/webhook", post(parse_update))
        .with_state(Arc::new(Mutex::new(queue)))
}
async fn parse_update(
    State(state): State<Arc<Mutex<AsyncQueue<NoTls>>>>,
    Json(update): Json<Update>,
) -> axum::response::Result<()> {
    log::info!("New update -> {:#?}", update);
    UpdateProcessor::run(&update, state).await?;
    Ok(())
}

impl IntoResponse for BotError {
    fn into_response(self) -> axum::response::Response {
        // Customize error response here
        (StatusCode::BAD_REQUEST, "Error processing update").into_response()
    }
}
