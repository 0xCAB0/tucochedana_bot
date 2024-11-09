use chrono::{DateTime, Utc};
use frankenstein::reqwest::{Client, StatusCode};

use crate::{BotError, API_URL};

pub struct TuCocheDanaClient {
    client: Client,
    base_url: String,
}

impl TuCocheDanaClient {
    pub async fn new(url: Option<String>) -> Self {
        let client = frankenstein::reqwest::ClientBuilder::new().build().unwrap();
        let base_url = url.unwrap_or(API_URL.to_string());
        TuCocheDanaClient { client, base_url }
    }

    pub async fn is_vehicle_found(&self, plate: &str) -> Result<DateTime<Utc>, BotError> {
        let result = self
            .client
            .get(&self.base_url)
            .query(&[("matricula", &plate)])
            .send()
            .await?;

        match result.status() {
            StatusCode::OK => Ok(Utc::now()),
            code => {
                let test = result.text().await?;
                Err(BotError::TuCocheDanaError(code, test))
            }
        }
    }
}

#[cfg(test)]
mod tu_coche_dana_client_tests {
    use super::*;
    #[tokio::test]
    async fn test_is_vehicle_found_ok() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::UrlEncoded(
                "matricula".to_string(),
                "ABC123".to_string(),
            ))
            .with_status(200)
            .with_body("hello")
            .create();
        let client = TuCocheDanaClient::new(Some(server.url())).await;
        let result = client.is_vehicle_found("ABC123").await;

        eprintln!("{:#?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_is_vehicle_found_not_found() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", "/")
            .match_query(mockito::Matcher::UrlEncoded(
                "matricula".to_string(),
                "XYZ789".to_string(),
            ))
            .with_status(404)
            .with_body("Vehicle not found")
            .create();

        let client = TuCocheDanaClient::new(Some(server.url())).await;
        let result = client.is_vehicle_found("XYZ789").await;

        match result {
            Err(BotError::TuCocheDanaError(status, message)) => {
                assert_eq!(status, StatusCode::NOT_FOUND);
                assert_eq!(message, "Vehicle not found");
            }
            _ => panic!("Expected BotError::TuCocheDanaError with NOT_FOUND status"),
        }
    }
}
