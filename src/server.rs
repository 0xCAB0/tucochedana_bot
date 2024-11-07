use std::net::SocketAddr;

use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use frankenstein::Update;

use crate::update_handler::process_update_task::ProcessUpdateTask;

pub fn app() -> Router {
    Router::new()
        .route("/", get(handle_root))
        .route("/webhook", post(parse_update))
}

async fn handle_root(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> String {
    format!("Hello! Your IP address is: {}", addr.ip())
}

async fn parse_update(update: Json<Update>) -> (StatusCode, Json<()>) {
    log::info!("New update {:?}", update);
    ProcessUpdateTask::new(update.0);
    (StatusCode::OK, Json(()))
}
