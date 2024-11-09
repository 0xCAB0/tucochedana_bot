use crate::BotError;

use crate::update_handler::process_update::UpdateProcessor;

const HELP_TEXT: &str = include_str!("../../../../resources/help.md");

impl UpdateProcessor {
    pub async fn help_menu(&self) -> Result<(), BotError> {
        // let rows: Vec<Vec<(&str, &str)>> = vec![vec![
        //     ("Soporte", "https://t.me/horus_soporte"),
        //     ("Seguimiento", "https://t.me/horus_seguimiento"),
        // ]];
        let rows = vec![vec![("⬅️ Back", "/start_back")]];

        let rows = Self::texts_to_buttons(rows, false);

        self.api
            .edit_or_send_message(self.chat.id, self.message_id, HELP_TEXT, rows)
            .await?;
        Ok(())
    }
}
