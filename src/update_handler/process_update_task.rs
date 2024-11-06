use std::str::FromStr;

use crate::db::model::client_state::ClientState;
use crate::db::{model::chat::Chat, Repo}; //Will change BotDbError

use crate::tasks::fetch::FetchTask;
use crate::telegram::client::ApiClient;
use crate::BotError;

use fang::asynk::async_queue::AsyncQueueable;
use fang::serde::{Deserialize, Serialize};
use fang::{async_trait, typetag, AsyncRunnable, FangError};

use frankenstein::{
    InlineKeyboardMarkup, MaybeInaccessibleMessage, Message, Update, UpdateContent,
};

use super::command::Command;
use typed_builder::TypedBuilder;

pub const SELECT_COMMAND_TEXT: &str = "Seleccione un comando";

/// Runs Fang's tasks async
#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "fang::serde")]
pub struct ProcessUpdateTask {
    update: Update,
}

impl ProcessUpdateTask {
    pub fn new(update: Update) -> Self {
        Self { update }
    }
    async fn remove_tasks(mut profiles: String) -> Result<(), FangError> {
        let repo = Repo::repo().await?;

        profiles.pop();

        for profile_id in profiles.split(',') {
            let profile_id: String = profile_id.to_string();
            repo.delete_tasks_by_chat_id(&profile_id).await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TaskToManage {
    FetchTask(FetchTask),
    FetchTasks(Vec<FetchTask>),
    RemoveTasks(String),
    NoTask,
}

#[typetag::serde]
#[async_trait]
impl AsyncRunnable for ProcessUpdateTask {
    async fn run(&self, queueable: &mut dyn AsyncQueueable) -> Result<(), FangError> {
        let processor = match UpdateProcessor::create(&self.update).await {
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
                    self.update,
                    error
                );

                if let Err(err) = processor.revert_state().await {
                    log::error!("Failed to revert: {:?}", err);
                }
            }

            Ok(option) => match option {
                TaskToManage::FetchTasks(tasks) => {
                    for task in tasks {
                        queueable.schedule_task(&task).await?;
                    }
                }
                TaskToManage::FetchTask(task) => {
                    queueable.schedule_task(&task).await?;
                }

                TaskToManage::RemoveTasks(profiles) => {
                    Self::remove_tasks(profiles).await?;
                }

                TaskToManage::NoTask => (),
            },
        }

        Ok(())
    }

    fn task_type(&self) -> String {
        "process_update".to_string()
    }
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
}

impl UpdateProcessor {
    pub async fn create(update: &Update) -> Result<Self, BotError> {
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

        let chat = repo
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
            .build();

        Ok(processor)
    }

    pub async fn process(&self) -> Result<TaskToManage, BotError> {
        //self.send_typing().await?;

        if Command::Cancel == self.command {
            self.cancel(None).await?;
            return Ok(TaskToManage::NoTask);
        }

        match self.chat.state {
            ClientState::Initial => self.process_initial().await,

            ClientState::AddVehicle => {
                if let Command::UnknownCommand(_) = self.command {
                    //self.edit_profile_name().await?;
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

            Command::AddVehicle => {
                if let Command::UnknownCommand(_) = self.command {
                    self.add_vehicle_message().await?;
                }
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
}
