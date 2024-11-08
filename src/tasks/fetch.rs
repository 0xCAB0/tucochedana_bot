use crate::db::Repo;
use crate::telegram::client::ApiClient;

use crate::tucochedana::client::TuCocheDanaClient;
use crate::{BotError, FETCH_IN_MINUTES, MAX_RETRIES, TASK_NAME};

use chrono::{DateTime, Datelike, Timelike, Utc};
use fang::{
    async_trait, typetag, AsyncQueueable, AsyncRunnable, Deserialize, FangError, Scheduled,
    Serialize,
};

use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, TypedBuilder, Eq, PartialEq, Clone)]
#[serde(crate = "fang::serde")]
pub struct FetchTask {
    plate: String,
}

#[typetag::serde]
#[async_trait]
impl AsyncRunnable for FetchTask {
    async fn run(&self, _queueable: &mut dyn AsyncQueueable) -> Result<(), FangError> {
        let repo = Repo::repo().await?;

        let telegram = ApiClient::api_client().await;

        let vehicle = repo.get_vehicle(self.plate.as_str()).await?;

        if vehicle.subscribers_ids.is_none() {
            let err = format!("Running tasks for plate {} with no subscribers", self.plate);
            log::error!("{}", &err);
            return Err(BotError::FetchTaskError(err).into());
        }

        let subscribers = repo.get_subscriptions_from_vehicle(&self.plate).await?;

        if let Some(found_at_timestamp) = vehicle.found_at {
            for sub in subscribers {
                telegram
                    .send_message_without_reply(
                        sub,
                        format!(
                            "El coche {} se encontró el {}",
                            self.plate, // or `vehicle.plate` if accessing from `vehicle`
                            datetime_to_text(found_at_timestamp)
                        ),
                    )
                    .await?;
            }
            repo.delete_tasks_by_plate(&self.plate).await?;
            return Ok(());
        }

        let tu_coche_dana = TuCocheDanaClient::new().await;
        match tu_coche_dana
            .get_vehicle_by_plate(self.plate.to_string())
            .await
        {
            Ok(()) => {
                let found_at = chrono::Utc::now();
                repo.modify_found_at_vehicle(&self.plate, found_at).await?;
                for sub in subscribers {
                    telegram
                        .send_message_without_reply(
                            sub,
                            format!(
                                "El coche {} se encontró el {}",
                                self.plate, // or `vehicle.plate` if accessing from `vehicle`
                                datetime_to_text(found_at)
                            ),
                        )
                        .await?;
                }
                repo.delete_tasks_by_plate(&self.plate).await?;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    fn uniq(&self) -> bool {
        true
    }

    fn cron(&self) -> Option<Scheduled> {
        let expression = format!("* */{} * * * * *", *FETCH_IN_MINUTES);
        Some(Scheduled::CronPattern(expression))
    }

    fn task_type(&self) -> String {
        TASK_NAME.to_string()
    }
    fn max_retries(&self) -> i32 {
        *MAX_RETRIES
    }
    fn backoff(&self, attempt: u32) -> u32 {
        u32::pow(2, attempt)
    }
}

fn datetime_to_text(time: DateTime<Utc>) -> String {
    // Spanish names for days of the week
    let days = [
        "domingo",
        "lunes",
        "martes",
        "miércoles",
        "jueves",
        "viernes",
        "sábado",
    ];
    // Spanish names for months
    let months = [
        "enero",
        "febrero",
        "marzo",
        "abril",
        "mayo",
        "junio",
        "julio",
        "agosto",
        "septiembre",
        "octubre",
        "noviembre",
        "diciembre",
    ];

    // Get day of the week, day of the month, month, and year
    let weekday = days[time.weekday().num_days_from_sunday() as usize];
    let day = time.day();
    let month = months[(time.month() - 1) as usize];
    let year = time.year();
    let hour = time.hour();
    let minute = time.minute();

    // Format the date as a Spanish-readable string
    format!(
        "{}, {} de {} de {}, {:02}:{:02}",
        weekday, day, month, year, hour, minute
    )
}
