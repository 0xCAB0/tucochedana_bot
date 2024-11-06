use frankenstein::reqwest::{Client, StatusCode};

use crate::{BotError, API_URL};

struct _TuCocheDanaClient {
    client: Client,
}

impl _TuCocheDanaClient {
    fn _new() -> Self {
        let client = frankenstein::reqwest::ClientBuilder::new().build().unwrap();

        _TuCocheDanaClient { client }
    }
    async fn _get_vehicle_by_plate(&self, plate: String) -> Result<(), BotError> {
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
