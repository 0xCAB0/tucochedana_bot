use frankenstein::reqwest::{Client, StatusCode};

use crate::{BotError, API_URL};

pub struct TuCocheDanaClient {
    client: Client,
}

impl TuCocheDanaClient {
    pub async fn new() -> Self {
        let client = frankenstein::reqwest::ClientBuilder::new().build().unwrap();

        TuCocheDanaClient { client }
    }

    pub async fn get_vehicle_by_plate(&self, plate: String) -> Result<(), BotError> {
        let result = self
            .client
            .get(API_URL.to_string())
            .query(&[("matricula", &plate)])
            .send()
            .await?;

        match result.status() {
            StatusCode::OK => Ok(()),
            code => {
                let test = result.text().await?;
                Err(BotError::TuCocheDanaError(code, test))
            }
        }
    }
}
