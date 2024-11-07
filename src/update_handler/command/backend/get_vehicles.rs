use crate::{update_handler::process_update::UpdateProcessor, BotError};

impl UpdateProcessor {
    pub async fn get_vehicles(&self) -> Result<(), BotError> {
        Ok(())
    }
}
