use crate::db::Repo;
use crate::telegram::client::ApiClient;

use crate::tucochedana::client::TuCocheDanaClient;
use crate::{BotError, FETCH_IN_MINUTES, MAX_RETRIES, TASK_NAME};

use fang::{
    async_trait, typetag, AsyncQueueable, AsyncRunnable, Deserialize, FangError, Scheduled,
    Serialize,
};

use bon::Builder;

#[derive(Serialize, Deserialize, Debug, Builder, Eq, PartialEq, Clone)]
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

        let mut vehicle = repo.get_vehicle(self.plate.as_str()).await?;

        if vehicle.subscribers_ids.is_none() {
            let err = format!("Running tasks for plate {} with no subscribers", self.plate);
            log::error!("{}", &err);
            return Err(BotError::FetchTaskError(err).into());
        }

        let subscribers = repo
            .get_active_subscriptions_from_vehicle(&self.plate) // Only create tasks for active users
            .await?;

        if vehicle.found_at.is_some() {
            for sub in subscribers {
                telegram
                    .send_message_without_reply(
                        sub.id,
                        format!(
                            "El coche {} se encontró el {}",
                            self.plate,
                            vehicle.datetime_to_text()
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
                vehicle.found_at = Some(found_at);
                repo.modify_found_at_vehicle(&self.plate, found_at).await?;
                for sub in subscribers {
                    telegram
                        .send_message_without_reply(sub.id, vehicle.datetime_to_text())
                        .await?;
                }
                repo.delete_tasks_by_plate(&self.plate).await?;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }

    fn uniq(&self) -> bool {
        true //Solo una tarea por vehículo
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
