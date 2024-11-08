use std::str::{FromStr, SplitAsciiWhitespace};
use std::sync::Arc;

use crate::db::model::client_state::ClientState;
use crate::db::{model::chat::Chat, Repo};

use crate::tasks::fetch::FetchTask;
use crate::telegram::client::ApiClient;
use crate::BotError;

use super::command::Command;
use fang::{AsyncQueue, AsyncQueueable, NoTls};
use frankenstein::{
    InlineKeyboardMarkup, MaybeInaccessibleMessage, Message, Update, UpdateContent,
};
use tokio::sync::Mutex;
use typed_builder::TypedBuilder;

pub const SELECT_COMMAND_TEXT: &str = "Seleccione un comando";

#[derive(Debug, Clone)]
pub enum TaskToManage {
    FetchTask(FetchTask),
    FetchTasks(Vec<FetchTask>),
    RemoveTasks(String),
    NoTask,
}

/// Telegram's Update event handler
#[derive(TypedBuilder)]
pub struct UpdateProcessor {
    pub api: &'static ApiClient,
    pub repo: &'static Repo,
    pub text: String,
    pub callback_data: Option<String>,
    pub message_id: i32,
    pub inline_keyboard: Option<Box<InlineKeyboardMarkup>>,
    pub command: Command,
    pub chat: Chat,
    pub is_first: bool,
}

impl UpdateProcessor {
    pub fn get_parse_iterator(&self) -> SplitAsciiWhitespace<'_> {
        let mut iter = self
            .callback_data
            .as_ref()
            .unwrap()
            .split_ascii_whitespace();
        iter.next();
        iter
    }

    async fn create(update: &Update) -> Result<Self, BotError> {
        let repo = Repo::repo().await?;
        let api = ApiClient::api_client().await;

        let (message, callback_data): (&Message, Option<String>) = match &update.content {
            UpdateContent::CallbackQuery(callback) => {
                match &callback.message {
                    None => {
                        // Log or handle the case where there is no message attached
                        panic!("No message attached to the callback query");
                    }
                    Some(MaybeInaccessibleMessage::Message(message)) => {
                        // Valid message case
                        (
                            message,
                            Some(callback.data.as_ref().unwrap().clone()), // Safe unwrap since we know there is data
                        )
                    }
                    Some(MaybeInaccessibleMessage::InaccessibleMessage(inaccessible_message)) => {
                        // Handle the inaccessible message case
                        // Deberíamos paniquear ?
                        panic!(
                            "Inaccessible message. Chat: {}, Message ID: {}",
                            inaccessible_message.chat.id, inaccessible_message.message_id
                        );
                    }
                }
            }
            UpdateContent::Message(message) => {
                let error = message.text.is_none() && message.successful_payment.is_none();

                if error {
                    log::error!("Update doesn't contain any text {:?}", message);
                    return Err(BotError::UpdateNotMessage("no text".to_string()));
                }

                (message, None)
            }
            _ => {
                log::error!("Update is not a message or callback {:?}", update);

                return Err(BotError::UpdateNotMessage("no message".to_string()));
            }
        };

        let text = message.text.as_ref().unwrap();

        let chat_id: i64 = message.chat.id;
        let user = message.from.as_ref().unwrap().clone();

        let username = match user.username {
            Some(name) => format!("@{}", name),
            None => user.first_name,
        };

        let (chat, is_first) = repo
            .find_or_create_chat(&chat_id, user.id, &username, &user.language_code)
            .await?;

        let command = match callback_data.as_ref() {
            Some(callback) => {
                let command = callback.split_ascii_whitespace().next().unwrap();
                Command::from_str(command).unwrap()
            }
            None => Command::from_str(text).unwrap(),
        };

        let keyboard = message.reply_markup.clone();

        let processor = Self::builder()
            .repo(repo)
            .api(api)
            .message_id(message.message_id)
            .text(text.clone())
            .callback_data(callback_data)
            .chat(chat)
            .command(command)
            .inline_keyboard(keyboard)
            .is_first(is_first)
            .build();

        Ok(processor)
    }

    pub async fn run(
        update: &Update,
        queue: Arc<Mutex<AsyncQueue<NoTls>>>,
    ) -> Result<(), BotError> {
        let processor = match UpdateProcessor::create(update).await {
            Ok(processor) => processor,
            Err(err) => {
                log::error!("Failed to initialize the processor {:?}", err);
                return Ok(());
            }
        };

        match processor.process().await {
            Err(error) => {
                log::error!(
                    "Failed to process the update {:?} - {:?}. Reverting...",
                    update,
                    error
                );

                if let Err(err) = processor.revert_state().await {
                    log::error!("Failed to revert: {:?}", err);
                }
            }

            Ok(option) => match option {
                TaskToManage::FetchTasks(tasks) => {
                    for task in tasks {
                        queue.try_lock().unwrap().schedule_task(&task).await?;
                    }
                }
                TaskToManage::FetchTask(task) => {
                    queue.try_lock().unwrap().schedule_task(&task).await?;
                }

                TaskToManage::RemoveTasks(subscribers) => {
                    Self::remove_tasks(subscribers).await?;
                }

                TaskToManage::NoTask => (),
            },
        }

        Ok(())
    }

    async fn process(&self) -> Result<TaskToManage, BotError> {
        if Command::Cancel == self.command {
            self.cancel(None).await?;
            return Ok(TaskToManage::NoTask);
        }

        match self.chat.state {
            ClientState::Initial => self.process_initial().await,

            ClientState::AddVehicle => {
                if let Command::UnknownCommand(_) = self.command {
                    self.add_vehicle().await?;
                }
                Ok(TaskToManage::NoTask)
            }
        }
    }

    async fn process_initial(&self) -> Result<TaskToManage, BotError> {
        match &self.command {
            Command::Help => {
                self.help_menu().await?;
                Ok(TaskToManage::NoTask)
            }

            Command::Start => {
                self.start_message(SELECT_COMMAND_TEXT).await?;
                Ok(TaskToManage::NoTask)
            }

            Command::StartBack => {
                self.start_message(SELECT_COMMAND_TEXT).await?;
                Ok(TaskToManage::NoTask)
            }

            Command::AddVehicleMessage => {
                self.add_vehicle_promt().await?;
                Ok(TaskToManage::NoTask)
            }

            Command::MyAddedVehicles => {
                self.get_vehicles().await?;
                Ok(TaskToManage::NoTask)
            }

            Command::StartFetch => self.start_fetch().await,

            Command::StopFetch => self.stop_fetch().await,

            Command::UnknownCommand(string) => {
                self.unknown_command(string).await?;
                Ok(TaskToManage::NoTask)
            }
            _ => Ok(TaskToManage::NoTask),
        }
    }

    async fn remove_tasks(mut subscribers: String) -> Result<(), BotError> {
        let repo = Repo::repo().await?;

        subscribers.pop();

        for chat_id in subscribers.split(',') {
            repo.delete_tasks_by_chat_id(chat_id).await?;
        }

        Ok(())
    }
}