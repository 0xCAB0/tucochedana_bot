use crate::{
    tasks::fetch::FetchTask,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn start_fetch(&self) -> Result<TaskToManage, BotError> {
        //Set chat as active
        self.repo.modify_active_chat(&self.chat.id, true).await?;

        let task = FetchTask::builder().chat_id(self.chat.id).build();

        //For all subscribed vehicles, try create a fetch Task

        //The task should handle notifying all the subscribed users

        Ok(TaskToManage::FetchTask(task))
    }
}
