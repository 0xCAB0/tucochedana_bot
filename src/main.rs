use tu_coche_dana_bot::{telegram::client::ApiClient, WEBHOOK_URL};

#[tokio::main]
async fn main() {
    // Telegram conf: `setjoingroups Disable` Hacer esto antes de correrlo en produccion
    dotenvy::dotenv().ok();
    // Logger
    pretty_env_logger::init_timed();

    let client = ApiClient::api_client().await.clone();
    let _ = client.set_webhook(&WEBHOOK_URL).await;
}
