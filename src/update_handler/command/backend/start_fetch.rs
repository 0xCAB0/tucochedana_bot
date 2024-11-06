use crate::{
    tasks::fetch::FetchTask,
    update_handler::process_update_task::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn start_fetch(&self) -> Result<TaskToManage, BotError> {
        let profile_id = 100i64;
        let task = FetchTask::builder().chat_id(profile_id).build();
        Ok(TaskToManage::FetchTask(task))
    }
}
