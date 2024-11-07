use crate::db::Repo;
use crate::telegram::client::ApiClient;

use crate::{FETCH_IN_MINUTES, MAX_RETRIES};

use fang::{
    async_trait, typetag, AsyncQueueable, AsyncRunnable, Deserialize, FangError, Scheduled,
    Serialize,
};

use typed_builder::TypedBuilder;

#[derive(Serialize, Deserialize, Debug, TypedBuilder, Eq, PartialEq, Clone)]
#[serde(crate = "fang::serde")]
pub struct FetchTask {
    chat_id: i64,
}

#[typetag::serde]
#[async_trait]
impl AsyncRunnable for FetchTask {
    async fn run(&self, queueable: &mut dyn AsyncQueueable) -> Result<(), FangError> {
        // Here we should do one deliver.
        let repo = Repo::repo().await?;

        let _telegram = ApiClient::api_client().await;

        let chat_raw = repo.get_chat(&self.chat_id).await;

        match chat_raw {
            Ok(chat) if !chat.active => {
                // `chat` is inactive, remove the task and return early
                queueable.remove_task_by_metadata(self).await?;
                Ok(())
            }
            Ok(_chat) => {
                // `chat` is active, proceed with the chat result
                // You can now use `chat` here as needed
                Ok(())
            }
            Err(err) => {
                // Handle the error case by removing the task and returning the error
                queueable.remove_task_by_metadata(self).await?;
                Err(err.into())
            }
        }
    }

    fn uniq(&self) -> bool {
        true
    }

    fn cron(&self) -> Option<Scheduled> {
        let expression = format!("* */{} * * * * *", *FETCH_IN_MINUTES);
        Some(Scheduled::CronPattern(expression))
    }

    fn task_type(&self) -> String {
        "scheduled_fetch".to_string()
    }
    fn max_retries(&self) -> i32 {
        *MAX_RETRIES
    }
    fn backoff(&self, attempt: u32) -> u32 {
        u32::pow(2, attempt)
    }
}
