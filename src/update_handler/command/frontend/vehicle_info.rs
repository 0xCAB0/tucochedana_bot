use crate::{
    db::model::vehicle::{self, Vehicle},
    update_handler::process_update::UpdateProcessor,
    BotError,
};

const VEHICLE_INFO: &str = "Información más reciente sobre el vehículo";

impl UpdateProcessor {
    pub async fn vehicle_info(&self) -> Result<(), BotError> {
        let mut iter = self.get_parse_iterator();

        let plate: &str = iter.next().unwrap();
        let vehicle: Vehicle = self.repo.get_vehicle(plate).await?;

        let rows = vec![vec![("🆗", "/create_edit_profile")]];

        let vec = Self::texts_to_buttons(rows, false);

        let text = format!("{VEHICLE_INFO}\n\n{}\n", vehicle.datetime_to_text());

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, &text, vec)
            .await?;
        Ok(())
    }
}
