use std::ops::Not;

use crate::{
    db::model::client_state::ClientState,
    tasks::fetch::FetchTask,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn add_vehicle(&self) -> Result<TaskToManage, BotError> {
        let plate = &self.text;

        let mut text = if self.repo.insert_vehicle(plate).await.is_ok() {
            //TODO: Si el vehículo es añadido por un usuario activo -> Lanzar task
            if self.chat.active {
                self.repo
                    .subscribe_chat_id_to_vehicle(plate, self.chat.id)
                    .await?;

                self.repo
                    .modify_state(&self.chat.id, ClientState::Initial)
                    .await?;
                self.api
                    .send_message_without_reply(self.chat.id, format!("Vehículo {plate} añadido, como tiene las alertas activas, le avisaremos si se registra"))
                    .await?;
                return Ok(TaskToManage::FetchTask(
                    FetchTask::builder().plate(plate.to_string()).build(),
                ));
            } else {
                format!("Vehículo {plate} añadido ✅")
            }
        } else {
            //TODO: ¿Deberíamos realizar alguna acción extra si ya está registrado?
            format!("El vehículo {plate} ya ha sido registrado por otro usuario, le añadiremos como interesado")
        };

        if self
            .repo
            .subscribe_chat_id_to_vehicle(plate, self.chat.id)
            .await
            .is_ok()
            .not()
        {
            text = format!("El vehículo {plate} ya ha sido añadido previamente 👀")
        };

        self.repo
            .modify_state(&self.chat.id, ClientState::Initial)
            .await?;

        self.start_message(Some(&text)).await?;

        Ok(TaskToManage::NoTask)
    }
}
