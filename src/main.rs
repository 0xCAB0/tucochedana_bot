use axum::{routing::post, Json, Router};
use frankenstein::Update;
use reqwest::StatusCode;
use tu_coche_dana_bot::{telegram::client::ApiClient, BotError, WEBHOOK_PORT, WEBHOOK_URL};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // Logger
    pretty_env_logger::init_timed();

    set_up_http_server(*WEBHOOK_PORT).await.unwrap();
    // Instanciate Telegram client
    let telegram = ApiClient::api_client().await;
    let _ = telegram.set_webhook(&WEBHOOK_URL, None).await;
}

async fn set_up_http_server(port: u32) -> Result<(), BotError> {
    let app = Router::new().route("/", post(parse_update));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    match axum::serve(listener, app).await {
        Ok(_) => Ok(()),
        Err(err) => Err(tu_coche_dana_bot::BotError::HttpError(err)),
    }
}

async fn parse_update(update: Json<Update>) -> (StatusCode, Json<()>) {
    log::info!("Update received {:#?}", update);
    (StatusCode::OK, Json(()))
}
