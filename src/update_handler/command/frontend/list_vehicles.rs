use crate::{update_handler::process_update::UpdateProcessor, BotError};

pub const VEHICLES_MENU_TEXT: &str = "Vehículos añadidos";
pub const ADD_VEHICLE: &str = "Añadir un vehículo";
pub const DELETE_EMOJI: &str = "❌";

impl UpdateProcessor {
    pub async fn get_vehicles(&self) -> Result<(), BotError> {
        let vehicles = match &self.chat.subscribed_vehicles {
            Some(vehicles) => self.repo.get_vehicles_from_subs_string(vehicles).await?,
            None => vec![],
        };

        let mut rows: Vec<Vec<(String, String)>> = vec![];

        vehicles.into_iter().enumerate().for_each(|(num, vehicle)| {
            rows.push(vec![]);

            rows[num].push((
                vehicle.plate.clone(),
                format!("/check_vehicle {}", vehicle.plate),
            ));

            // rows[num].push((
            //     DELETE_EMOJI.to_string(),
            //     format!("/delete_vehicle {}", vehicle.plate),
            // ));
        });
        rows.push(vec![(
            ADD_VEHICLE.to_string(),
            "/add_vehicle_message".to_string(),
        )]);
        rows.push(vec![("⬅️ Back".to_string(), "/start_back".to_string())]);

        let vec = Self::texts_to_buttons(rows, false);

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, VEHICLES_MENU_TEXT, vec)
            .await?;

        Ok(())
    }
}
