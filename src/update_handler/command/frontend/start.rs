use crate::BotError;

pub const START_OPTIONS_1: &str = "Añadir un vehículo";
pub const START_OPTIONS_1_2: &str = "Mis vehículos";
pub const START_OPTIONS_2: &str = "Activar alerta";
pub const START_OPTIONS_3: &str = "Desactivar alerta";
pub const START_OPTIONS_4: &str = "Ayuda";

pub const WELCOME_MESSAGE: &str = "¡Bienvenido!";

use crate::update_handler::process_update::UpdateProcessor;

impl UpdateProcessor {
    pub async fn start_message(&self, text: &str) -> Result<(), BotError> {
        //Handling new user

        if self.is_first {
            self.api
                .send_message_without_reply(self.chat.id, WELCOME_MESSAGE)
                .await?;
        }

        let alert_row = if !self.chat.active {
            (START_OPTIONS_2, "/start_fetch")
        } else {
            (START_OPTIONS_3, "/stop_fetch")
        };

        let rows = vec![
            vec![
                (START_OPTIONS_1, "/add_vehicle_message"),
                (START_OPTIONS_1_2, "/get_my_vehicles"),
            ],
            vec![alert_row],
            vec![(START_OPTIONS_4, "/help")],
        ];

        let vec = Self::texts_to_buttons(rows, false);

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, text, vec)
            .await?;

        Ok(())
    }
}
