use crate::{
    db::model::client_state::ClientState, update_handler::process_update::UpdateProcessor, BotError,
};

impl UpdateProcessor {
    pub async fn add_vehicle(&self) -> Result<(), BotError> {
        let plate = &self.text;

        // let text = if self.repo.insert_vehicle(plate).await.is_ok() {
        //     format!("Vehículo {plate} añadido")
        // } else {
        //     //TODO: ¿Deberíamos realizar alguna acción extra si ya está registrado?
        //     format!("El vehículo {plate} ya ha sido registrado por otro usuario, le añadiremos como interesado")
        // };

        self.repo.find_or_create_vehicle(plate).await?;

        let text = if self
            .repo
            .subscribe_chat_id_to_vehicle(plate, self.chat.id)
            .await
            .is_ok()
        {
            format!("Vehículo {plate} añadido")
        } else {
            format!("El vehículo {plate} ya ha sido añadido previamente")
        };

        self.repo
            .modify_state(&self.chat.id, ClientState::Initial)
            .await?;

        self.api
            .send_message_without_reply(self.chat.id, text)
            .await?;

        Ok(())
    }
}
