use crate::{update_handler::process_update::UpdateProcessor, BotError};

const VEHICLE_INFO: &str = "Información más reciente sobre el vehículo";

impl UpdateProcessor {
    pub async fn vehicle_info(&self) -> Result<(), BotError> {
        let mut iter = self.get_parse_iterator();
        let plate: &str = iter.next().unwrap();

        // Handling accessing unknow vehicle
        let vehicle = self.repo.find_or_create_vehicle(plate).await?;

        let rows = vec![vec![("⬅️ Back", "/get_my_vehicles")]];
        let vec = Self::texts_to_buttons(rows, false);
        let text = format!("{VEHICLE_INFO}\n\n{}\n", vehicle.datetime_to_text());

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, &text, vec)
            .await?;

        Ok(())
    }
}
