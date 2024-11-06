use crate::{update_handler::process_update_task::UpdateProcessor, BotError};

impl UpdateProcessor {
    pub async fn add_vehicle(&self) -> Result<(), BotError> {
        let mut iter = self
            .callback_data
            .as_ref()
            .unwrap()
            .split_ascii_whitespace();
        iter.next();
        let id_brand = iter.next().unwrap();
        Ok(())
    }

    pub async fn add_vehicle_message(&self) -> Result<(), BotError> {
        let plate = &self.text;
        Ok(())
    }
}
