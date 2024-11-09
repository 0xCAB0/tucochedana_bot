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
                            vehicle.found_at_to_text()
                        ),
                    )
                    .await?;
            }
            repo.delete_tasks_by_plate(&self.plate).await?;
            return Ok(());
        }

        let tu_coche_dana = TuCocheDanaClient::new(None).await;
        match tu_coche_dana.is_vehicle_found(&self.plate).await {
            Ok(found_at) => {
                vehicle.found_at = Some(found_at); //Mejor mantener vehiculo mutable o crear un objeto nuevo?
                repo.modify_found_at_vehicle(&self.plate, found_at).await?;
                for sub in subscribers {
                    telegram
                        .send_message_without_reply(sub.id, vehicle.found_at_to_text())
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

#[cfg(test)]
mod fetch_task_tests {
    // use super::*;
    // use crate::test::*;

    // #[tokio::test]
    // pub async fn test_fetch_task() {
    //     clear_database().await.unwrap();
    //     populate_database().await.unwrap();

    //     let db_controller = Repo::new_no_tls().await.unwrap();
    //     let connection = db_controller.get_connection().get().await.unwrap();

    //     let testing_plate = "GHI789";
    //     let testing_chat = 3;
    // }
}
