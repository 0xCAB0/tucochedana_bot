use crate::{
    db::model::vehicle::Vehicle,
    tasks::fetch::FetchTask,
    tucochedana::client::TuCocheDanaClient,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    fn sanitize_input(input: &str) -> Option<String> {
        if input.contains(',') {
            return None;
        }

        Some(String::from(input))
    }

    pub async fn add_vehicle(&self) -> Result<TaskToManage, BotError> {
        let Some(plate) = Self::sanitize_input(&self.text) else {
            self.add_vehicle_prompt(Some(
                "La matrícula introducida sigue un formato incorrecto, pruebe de nuevo",
            ))
            .await?;
            return Ok(TaskToManage::NoTask);
        };
        log::info!("Adding vehicle {plate}");
        let client = TuCocheDanaClient::new(None).await;

        let found_at = client.is_vehicle_found(&plate).await.ok();

        let vehicle = Vehicle::builder()
            .plate(plate.clone())
            .subscribers_ids(format!("{},", self.chat.id))
            .maybe_found_at(found_at)
            .build();

        if found_at.is_some() {
            self.api
                .send_message_without_reply(self.chat.id, vehicle.found_at_to_text())
                .await?;
            return Ok(TaskToManage::NoTask);
        }

        let text = if self.repo.insert_vehicle(vehicle).await.is_ok() {
            self.repo
                .append_subscription_to_chat(&plate, &self.chat.id)
                .await?;
            //Si el vehículo es añadido por un usuario activo -> Lanzar task
            if self.chat.active {
                self.repo.create_subscription(&plate, self.chat.id).await?;
                self.get_vehicles(Some(&format!("Vehículo {plate} añadido✅\ncomo tiene las alertas activas, le avisaremos si se registra")))
                    .await?;
                return Ok(TaskToManage::FetchTask(
                    FetchTask::builder().plate(plate).build(),
                ));
            } else {
                format!("Vehículo {plate} añadido ✅")
            }
        } else if self
            .repo
            .create_subscription(&plate, self.chat.id)
            .await
            .is_err()
        {
            format!("El vehículo {plate} ya ha sido añadido previamente 👀")
        } else {
            format!("El vehículo {plate} ya ha sido registrado por otro usuario, le añadiremos como interesado")
        };

        self.get_vehicles(Some(&text)).await?;

        Ok(TaskToManage::NoTask)
    }
}

#[cfg(test)]
mod add_vehicle_tests {
    use std::sync::Arc;

    use frankenstein::{Chat, Message, Update, UpdateContent, User};
    use tokio::sync::Mutex;

    use crate::db::{
        model::{self},
        Repo,
    };

    use super::*;

    #[tokio::test]
    async fn test_add_vehicle() {
        dotenvy::dotenv().ok();
        pretty_env_logger::init();

        let repo = Repo::repo().await.unwrap();
        let test_queue = Repo::create_testing_queue(repo, true).await.unwrap();

        let plate = "NUEVA789";
        let chat_id = 2;

        assert_eq!(
            1,
            repo.modify_state(&chat_id, model::client_state::ClientState::AddVehicle)
                .await
                .unwrap()
        );

        let db_chat = repo.get_chat(&chat_id).await.unwrap();

        let chat: Chat = Chat::builder()
            .id(db_chat.user_id as i64)
            .type_field(frankenstein::ChatType::Private)
            .username("Test".to_string())
            .first_name("Test".to_string())
            .last_name("Test Lastname".to_string())
            .build();

        let from = User::builder()
            .id(db_chat.user_id)
            .is_bot(false)
            .username(db_chat.username)
            .first_name("Test".to_string())
            .last_name("Test Lastname".to_string())
            .build();
        let message: Message = Message::builder()
            .message_id(1365)
            .date(crate::db::repo::db_tests::random_datetime().timestamp() as u64)
            .chat(chat)
            .from(from)
            .text(plate)
            .build();

        let content: UpdateContent = UpdateContent::Message(message);
        let update: Update = Update::builder().update_id(10000).content(content).build();

        match UpdateProcessor::run(&update, Arc::new(Mutex::new(test_queue))).await {
            Ok(processor) => {
                log::info!("{:#?}", processor);
            }
            Err(BotError::TelegramError(_)) => {}
            Err(err) => {
                log::error!("{:#?}", err);
                panic!()
            }
        };

        let _db_chat = repo.get_chat(&chat_id).await.unwrap();

        // assert_eq!(db_chat.state, ClientState::Initial);
        // assert!(db_chat.subscribed_vehicles.is_some_and(|subs| subs
        //     .split(',')
        //     .any(|subbed_vehicle| subbed_vehicle == plate)));
    }
}
