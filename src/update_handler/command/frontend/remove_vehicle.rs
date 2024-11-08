use crate::{update_handler::process_update::UpdateProcessor, BotError};

impl UpdateProcessor {
    pub async fn remove_vehicle(&self) -> Result<(), BotError> {
        Ok(())
    }
}
