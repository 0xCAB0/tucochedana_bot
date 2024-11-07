use crate::BotError;

use crate::update_handler::process_update::UpdateProcessor;

const HELP_TEXT: &str = include_str!("../../../../resources/help.md");

impl UpdateProcessor {
    pub async fn help_menu(&self) -> Result<(), BotError> {
        let rows: Vec<Vec<(&str, &str)>> = vec![vec![
            ("Soporte", "https://t.me/horus_soporte"),
            ("Seguimiento", "https://t.me/horus_seguimiento"),
        ]];
        let rows = Self::texts_to_buttons(rows, false);

        let chunks: Vec<&str> = HELP_TEXT
            .as_bytes()
            .chunks(1000)
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            if i == chunks.len() - 1 {
                self.api
                    .send_message_with_buttons(self.chat.id, chunk, rows.clone())
                    .await?;
            } else {
                // Otherwise, send a regular message
                self.api
                    .send_message_without_reply(self.chat.id, chunk.to_string())
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn help_menu_unpaid(&self) -> Result<(), BotError> {
        //Mandar video de introducción
        let rows: Vec<Vec<(&str, &str)>> = vec![vec![
            ("Soporte", "https://t.me/horus_soporte"),
            ("Seguimiento", "https://t.me/horus_seguimiento"),
        ]];
        let rows = Self::texts_to_buttons(rows, false);
        self.api
            .send_message_with_buttons(self.chat.id, HELP_TEXT, rows)
            .await?;

        Ok(())
    }
}
