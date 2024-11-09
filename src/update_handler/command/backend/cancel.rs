use crate::update_handler::process_update::UpdateProcessor;
use crate::BotError;

impl UpdateProcessor {
    pub async fn cancel(&self, custom_message: Option<String>) -> Result<(), BotError> {
        self.return_to_initial().await?;

        let text: String = match custom_message {
            Some(message) => message,
            None => "Your operation was canceled".to_string(),
        };
        self.send_message(&text).await
        //self.start_message(None).await
    }

    pub async fn revert_state(&self) -> Result<(), BotError> {
        self.cancel(None).await
    }
}
