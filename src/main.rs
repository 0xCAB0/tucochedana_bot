use tu_coche_dana_bot::{
    server::app, telegram::client::ApiClient, SERVER_PORT, WEBHOOK_CERT, WEBHOOK_PORT, WEBHOOK_URL,
};

#[tokio::main]
async fn main() {
    // Environment variables
    dotenvy::dotenv().ok();
    // Logger
    pretty_env_logger::init_timed();

    // Webhook setup
    let telegram = ApiClient::api_client().await;
    let webhook = match *WEBHOOK_PORT {
        443 | 80 => format!("{}/webhook", *WEBHOOK_URL), //Debe estar bien formateado (http o https)
        _ => format!("{}:{}/webhook", *WEBHOOK_URL, *WEBHOOK_PORT),
    };
    let response = telegram
        .set_webhook(&webhook, None, WEBHOOK_CERT.clone())
        .await
        .unwrap();
    if response.ok && response.result {
        log::info!("Setted Telegram webhook at URL {}", webhook);
    } else {
        log::error!("{:?}", response.description);
    }
    // Http Server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", *SERVER_PORT))
        .await
        .unwrap();
    log::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app()).await.unwrap();
}
