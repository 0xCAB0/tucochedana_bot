use crate::{update_handler::process_update::UpdateProcessor, BotError};

impl UpdateProcessor {
    pub async fn add_vehicle(&self) -> Result<(), BotError> {
        let plate = self.chat.selected_text.as_ref().unwrap();

        let text = if self.repo.insert_vehicle(plate).await.is_ok() {
            format!("Vehículo {plate} añadido")
        } else {
            //TODO: ¿Deberíamos realizar alguna acción extra si ya está registrado?
            format!("El vehículo {plate} ya ha sido registrado por otro usuario, le añadiremos como interesado")
        };

        self.repo
            .subscribe_chat_id_to_vehicle(plate, self.chat.id)
            .await?;

        let keyboard = *self.inline_keyboard.clone().unwrap();

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, &text, keyboard)
            .await?;

        Ok(())
    }
}
