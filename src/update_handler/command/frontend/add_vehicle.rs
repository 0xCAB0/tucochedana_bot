use crate::{
    db::model::client_state::ClientState, update_handler::process_update_task::UpdateProcessor,
    BotError,
};

pub const ADD_BRAND_MESSAGE_TEXT: &str =
    "Escribe la matrícula del vehículo del que deseas recibir alertas";

impl UpdateProcessor {
    pub async fn add_vehicle_promt(&self) -> Result<(), BotError> {
        self.api
            .send_message_without_reply(self.chat.id, ADD_BRAND_MESSAGE_TEXT.to_owned())
            .await?;

        self.repo
            .modify_state(&self.chat.id, ClientState::AddVehicle)
            .await?;

        Ok(())
    }
}
