use crate::{
    db::model::client_state::ClientState, update_handler::process_update::UpdateProcessor, BotError,
};

pub const ADD_VEHICLE_TEXT: &str =
    "Escribe la matrícula del vehículo del que deseas recibir alertas o /cancel para cancelar";

impl UpdateProcessor {
    pub async fn add_vehicle_prompt(&self, text: Option<&str>) -> Result<(), BotError> {
        let text = match text {
            Some(t) => t,
            None => ADD_VEHICLE_TEXT,
        };
        self.api
            .send_message_without_reply(self.chat.id, text)
            .await?;

        self.repo
            .modify_state(&self.chat.id, ClientState::AddVehicle)
            .await?;

        Ok(())
    }
}
