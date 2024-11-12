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
                    .send_message_without_reply(sub.id, vehicle.found_at_to_text())
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

    use chrono::Utc;

    use super::*;
    use crate::test::*;

    #[tokio::test]
    async fn test_fetch_task() {
        let db_controller = Repo::new_for_test().await.unwrap();
        populate_database(&db_controller).await.unwrap();
        let connection = db_controller.get_connection().get().await.unwrap();

        let testing_plate = String::from("MATRICULA1");
        let testing_plate_found = String::from("MATRICULA2");
        let _testing_plate_really_found = String::from("");
        let testing_chat = 1334_i64;

        // Add subscribed, active user
        connection
        .execute(
            "INSERT INTO chats (id, user_id, username, language_code, subscribed_vehicles, active)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT (id) DO NOTHING",
            &[
                &testing_chat,
                &345678_u64.to_le_bytes().to_vec(),
                &"user3",
                &Some("es".to_string()),
                &Some(format!("{},", testing_plate)),
                &true
            ],
        )
        .await.unwrap();

        // Add found vehicle
        connection
            .execute(
                "INSERT INTO vehicles (plate, subscribers_ids, found_at)
VALUES ($1, $2, $3)
ON CONFLICT (plate) DO NOTHING",
                &[
                    &testing_plate,
                    &format!("{},1,3,", testing_chat),
                    &Utc::now(),
                ],
            )
            .await
            .unwrap();

        // Add not found vehicle
        connection
            .execute(
                "INSERT INTO vehicles (plate, subscribers_ids)
VALUES ($1, $2)
ON CONFLICT (plate) DO NOTHING",
                &[&testing_plate_found, &format!("{},", testing_chat)],
            )
            .await
            .unwrap();

        //TODO: Add a vehicle as not found and make sure it's already found (API)

        let mut fake_queue = create_mock_queue().await.unwrap();
        let tasks = vec![
            FetchTask::builder().plate(testing_plate).build(),
            FetchTask::builder().plate(testing_plate_found).build(),
        ];

        for task in tasks {
            match task.run(&mut fake_queue).await.err() {
                Some(err) if err.description.contains("chat not found") => (), // Ignore invalid chat_id
                Some(err) => {
                    eprintln!("{:#?}", err);
                    unreachable!()
                }
                None => (),
            }
        }
    }
}
