use tu_coche_dana_bot::{server::app, telegram::client::ApiClient, WEBHOOK_PORT, WEBHOOK_URL};

#[tokio::main]
async fn main() {
    // Environment variables
    dotenvy::dotenv().ok();
    // Logger
    pretty_env_logger::init_timed();
    // Http Server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", *WEBHOOK_PORT))
        .await
        .unwrap();

    log::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
    // Instanciate Telegram client
    let telegram = ApiClient::api_client().await;
    let _ = telegram.set_webhook(&WEBHOOK_URL, None).await;
}
