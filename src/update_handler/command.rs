use frankenstein::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::{db::model::client_state::ClientState, BotError, BOT_NAME};
use std::str::FromStr;

use super::process_update::UpdateProcessor;

/// Avalible bots commands as a Enum
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Command {
    AddVehicle,
    AddVehicleMessage,
    MyAddedVehicles,
    VehicleInfo,
    StartFetch,
    StopFetch,
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
            "/vehicle_info" => Command::VehicleInfo,
            "/add_vehicle_message" => Command::AddVehicleMessage,
            "/get_my_vehicles" => Command::MyAddedVehicles,
            "/start_fetch" => Command::StartFetch,
            "/stop_fetch" => Command::StopFetch,
            _ => Command::UnknownCommand(command_str.to_string()),
        };

        Ok(result)
    }
}

pub mod backend {
    pub mod add_vehicle;
    pub mod cancel;
    pub mod start_fetch;
    pub mod stop_fetch;
}
pub mod frontend {
    pub mod add_vehicle;
    pub mod help;
    pub mod list_vehicles;
    pub mod start;
    pub mod vehicle_info;
}

impl UpdateProcessor {
    pub async fn return_to_initial(&self) -> Result<(), BotError> {
        self.repo
            .modify_state(&self.chat.id, ClientState::Initial)
            .await?;
        Ok(())
    }

    pub async fn unknown_command(&self, command: &str) -> Result<(), BotError> {
        self.cancel(Some(format!(
            "Comando '{}' desconocido. Ejecuta /start para ver los comandos disponibles",
            command
        )))
        .await?;
        Ok(())
    }

    pub fn texts_to_buttons<S: AsRef<str> + Into<String>, S2: AsRef<str> + Into<String>>(
        rows: Vec<Vec<(S, S2)>>,
        url: bool,
    ) -> InlineKeyboardMarkup {
        if url {
            let vec = rows
                .into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|(text, url)| {
                            InlineKeyboardButton::builder().url(url).text(text).build()
                        })
                        .collect()
                })
                .collect();

            return InlineKeyboardMarkup::builder().inline_keyboard(vec).build();
        }

        let vec = rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|(text, command)| {
                        InlineKeyboardButton::builder()
                            .callback_data(command)
                            .text(text)
                            .build()
                    })
                    .collect()
            })
            .collect();

        InlineKeyboardMarkup::builder().inline_keyboard(vec).build()
    }

    pub async fn send_message(&self, text: &str) -> Result<(), BotError> {
        let text_with_username = format!("Hola, {}!\n{}", self.chat.username, text);

        self.api
            .send_message(self.chat.id, self.message_id, text_with_username)
            .await?;

        Ok(())
    }

    async fn _send_typing(&self) -> Result<(), BotError> {
        self.api.send_typing(self.chat.id).await?;
        Ok(())
    }
}
