use crate::{update_handler::process_update::UpdateProcessor, BotError};

impl UpdateProcessor {
    pub async fn remove_vehicle(&self) -> Result<(), BotError> {
        // 1. Remove it from chat.subscribed_vehicles
        // 2. Remove chat from vehicle subscribers_ids
        // 3. If vehicle subscribers_ids is empty -> Remove task & delete vehicle

        Ok(())
    }
}
