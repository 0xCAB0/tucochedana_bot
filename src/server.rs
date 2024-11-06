use axum::{http::StatusCode, routing::post, Json, Router};
use frankenstein::Update;

pub fn app() -> Router {
    Router::new().route("/", post(parse_update))
}
async fn parse_update(update: Json<Update>) -> (StatusCode, Json<Update>) {
    log::info!("Update received {:#?}", update);
    (StatusCode::OK, update)
}
