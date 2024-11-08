use crate::{
    tasks::fetch::FetchTask,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn start_fetch(&self) -> Result<TaskToManage, BotError> {
        let task = FetchTask::builder().chat_id(self.chat.id).build();
        Ok(TaskToManage::FetchTask(task))
    }
}
