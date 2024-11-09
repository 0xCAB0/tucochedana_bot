use crate::{
    db::model::vehicle::Vehicle, update_handler::process_update::UpdateProcessor, BotError,
};

const VEHICLE_INFO: &str = "Información más reciente sobre el vehículo";

impl UpdateProcessor {
    pub async fn vehicle_info(&self) -> Result<(), BotError> {
        let mut iter = self.get_parse_iterator();

        // TODO: gestionar el estado si el comando no le han pasado una matricula de un vehiculo.
        // Preguntar otra vez por la matricula
        let plate: &str = iter.next().unwrap();
        let vehicle: Vehicle = self.repo.get_vehicle(plate).await?;

        let rows = vec![vec![("⬅️ Back", "/get_my_vehicles")]];

        let vec = Self::texts_to_buttons(rows, false);

        let text = format!("{VEHICLE_INFO}\n\n{}\n", vehicle.datetime_to_text());

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, &text, vec)
            .await?;
        Ok(())
    }
}
