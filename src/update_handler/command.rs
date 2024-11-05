use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{
    BotError, BOT_NAME}
;
use std::str::FromStr;


/// Avalible bots commands as a Enum
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Command {
    AddVehicle,
    MyAddedVehicles,
    FindVehicle,
    Help,
    Start,
    StartBack,
    Cancel,
    UnknownCommand(String),
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command_str = s.replace(&*BOT_NAME, "");

        let result = match command_str.trim() {
            "/start" => Command::Start,
            "/help" => Command::Help,
            "/start_back" => Command::StartBack,
            "/add_vehicle" => Command::AddVehicle,
            "/my_added_vehicles" => Command::AddVehicle,
            "/find_vehicle" => Command::FindVehicle,
            _ => Command::UnknownCommand(command_str.to_string()),
        };

        Ok(result)
    }
}

pub mod backend {
    pub mod cancel;
}
pub mod frontend {
    pub mod start;
    pub mod add_vehicle;
}

// impl UpdateProcessor {
//     pub async fn unknown_command(&self, command: &str) -> Result<(), BotError> {
//         self.cancel(Some(format!(
//             "Comando '{}' desconocido. Ejecuta /start para ver los comandos disponibles",
//             command
//         )))
//         .await?;
//         Ok(())
//     }

//     /*async fn not_number_message(&self) -> Result<(), BotError> {
//         self.cancel(Some(
//             "That's not a positive number in the range. The command was cancelled".to_string(),
//         ))
//         .await
//     }*/

//     pub fn texts_to_buttons<S: AsRef<str> + Into<String>, S2: AsRef<str> + Into<String>>(
//         rows: Vec<Vec<(S, S2)>>,
//         url: bool,
//     ) -> InlineKeyboardMarkup {
//         if url {
//             let vec = rows
//                 .into_iter()
//                 .map(|row| {
//                     row.into_iter()
//                         .map(|(text, url)| {
//                             InlineKeyboardButton::builder().url(url).text(text).build()
//                         })
//                         .collect()
//                 })
//                 .collect();

//             return InlineKeyboardMarkup::builder().inline_keyboard(vec).build();
//         }

//         let vec = rows
//             .into_iter()
//             .map(|row| {
//                 row.into_iter()
//                     .map(|(text, command)| {
//                         InlineKeyboardButton::builder()
//                             .callback_data(command)
//                             .text(text)
//                             .build()
//                     })
//                     .collect()
//             })
//             .collect();

//         InlineKeyboardMarkup::builder().inline_keyboard(vec).build()
//     }

//     pub async fn send_message(&self, text: &str) -> Result<(), BotError> {
//         let text_with_username = format!("Hola, {}!\n{}", self.chat.username, text);

//         self.api
//             .send_message(self.chat.id, self.message_id, text_with_username)
//             .await?;

//         Ok(())
//     }

//     async fn send_typing(&self) -> Result<(), BotError> {
//         self.api.send_typing(self.chat.id).await?;
//         Ok(())
//     }

//     /*async fn not_valid_offset_message(&self) -> Result<(), BotError> {
//         self.cancel(Some(
//             "That's not a valid offset, it has to be a number in range [-11, 12].\n
//             If your timezone is UTC + 2 put 2, if you have UTC - 10 put -10, 0 if you have UTC timezone.\n
//             The command was cancelled"
//             .to_string(),
//         ))
//         .await?;

//         Ok(())
//     }*/

//     /*async fn not_time_message(&self) -> Result<(), BotError> {
//         self.cancel(Some(
//             "That's not a well formatted time, it has to be formatted with this format `hour:minutes` being hour a number in range [0,23]
//             and minutes a number in range [0,59]. The command was cancelled"
//             .to_string(),
//         ))
//         .await
//     }*/

//     /*fn parse_time(hour_or_minutes: &str, max_range: i8, min_range: i8) -> Result<i8, ()> {
//         match hour_or_minutes.parse::<i8>() {
//             Ok(number) => {
//                 if !(min_range..=max_range).contains(&number) {
//                     Err(())
//                 } else {
//                     Ok(number)
//                 }
//             }
//             Err(_) => Err(()),
//         }
//     }*/
// }
