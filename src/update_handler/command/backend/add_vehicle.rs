use crate::{
    db::model::{client_state::ClientState, vehicle::Vehicle},
    tasks::fetch::FetchTask,
    tucochedana::client::TuCocheDanaClient,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn add_vehicle(&self) -> Result<TaskToManage, BotError> {
        let plate = String::from(&self.text);
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
            //Si el vehículo es añadido por un usuario activo -> Lanzar task
            if self.chat.active {
                self.repo
                    .subscribe_chat_id_to_vehicle(&plate, self.chat.id)
                    .await?;

                self.repo
                    .modify_state(&self.chat.id, ClientState::Initial)
                    .await?;
                self.api
                    .send_message_without_reply(self.chat.id, format!("Vehículo {plate} añadido✅\ncomo tiene las alertas activas, le avisaremos si se registra"))
                    .await?;
                return Ok(TaskToManage::FetchTask(
                    FetchTask::builder().plate(plate).build(),
                ));
            } else {
                format!("Vehículo {plate} añadido ✅")
            }
        } else if self
            .repo
            .subscribe_chat_id_to_vehicle(&plate, self.chat.id)
            .await
            .is_err()
        {
            format!("El vehículo {plate} ya ha sido añadido previamente 👀")
        } else {
            format!("El vehículo {plate} ya ha sido registrado por otro usuario, le añadiremos como interesado")
        };

        // Restaurar el estado

        self.repo
            .modify_state(&self.chat.id, ClientState::Initial)
            .await?;

        self.start_message(Some(&text)).await?;

        Ok(TaskToManage::NoTask)
    }
}
