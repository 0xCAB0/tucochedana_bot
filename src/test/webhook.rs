// Main reference https://core.telegram.org/bots/webhooks

use frankenstein::reqwest::Client;

use crate::telegram;

#[tokio::test]
async fn test_webhook_dispath() {
    let client = Client::new();
}
