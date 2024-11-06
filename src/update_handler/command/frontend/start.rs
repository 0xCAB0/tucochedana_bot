use crate::BotError;

pub const START_OPTIONS_1: &str = "Menu perfiles";
pub const START_OPTIONS_2: &str = "Activar perfiles";
pub const START_OPTIONS_3: &str = "Desactivar perfiles";
pub const START_OPTIONS_4: &str = "Ayuda";
pub const START_OPTIONS_5: &str = "Renovar suscripción";

use crate::update_handler::process_update_task::UpdateProcessor;

impl UpdateProcessor {
    pub async fn start_message(&self, text: &str) -> Result<(), BotError> {
        let rows = vec![
            vec![(START_OPTIONS_1, "/create_edit_profile")],
            vec![
                (START_OPTIONS_2, "/activate_profile_menu"),
                (START_OPTIONS_3, "/deactivate_profile_menu"),
            ],
            vec![(START_OPTIONS_4, "/help"), (START_OPTIONS_5, "/sub_menu")],
        ];

        let vec = Self::texts_to_buttons(rows, false);

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, text, vec)
            .await?;

        Ok(())
    }
}
