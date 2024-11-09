use crate::{
    db::model::client_state::ClientState, update_handler::process_update::UpdateProcessor, BotError,
};

pub const ADD_BRAND_MESSAGE_TEXT: &str =
    "Escribe la matrícula del vehículo del que deseas recibir alertas o /cancel para cancelar";

impl UpdateProcessor {
    pub async fn add_vehicle_prompt(&self) -> Result<(), BotError> {
        self.api
            .send_message_without_reply(self.chat.id, ADD_BRAND_MESSAGE_TEXT)
            .await?;

        self.repo
            .modify_state(&self.chat.id, ClientState::AddVehicle)
            .await?;

        Ok(())
    }
}
