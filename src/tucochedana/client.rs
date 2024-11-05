use frankenstein::BASE_API_URL;
use reqwest::{Client, StatusCode};

use crate::{BotError, API_URL};

struct TuCocheDanaClient {
    client: Client,
}

impl TuCocheDanaClient {
    fn new() -> Self {
        let client = reqwest::ClientBuilder::new().build().unwrap();

        TuCocheDanaClient { client }
    }
    async fn get_vehicle_by_plate(&self, plate: String) -> Result<(), BotError> {
        let result = self
            .client
            .get(&API_URL.to_string())
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
