// use crate::update_handler::process_update_task::UpdateProcessor;
// use crate::BotError;

// impl UpdateProcessor {
//     pub async fn cancel(&self, custom_message: Option<String>) -> Result<(), BotError> {

//         let text: String = match custom_message {
//             Some(message) => message,
//             None => "Your operation was canceled".to_string(),
//         };
//         self.send_message(&text).await
//     }

//     pub async fn revert_state(&self) -> Result<(), BotError> {
//         self.cancel(None).await
//     }
// }
