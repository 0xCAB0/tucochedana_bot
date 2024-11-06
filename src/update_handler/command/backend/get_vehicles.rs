use crate::{update_handler::process_update_task::UpdateProcessor, BotError};

impl UpdateProcessor {
    pub async fn get_vehicles(&self) -> Result<(), BotError> {
        Ok(())
    }
}
