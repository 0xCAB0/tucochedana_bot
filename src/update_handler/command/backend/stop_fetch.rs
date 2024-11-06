use crate::{
    update_handler::process_update_task::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn stop_fetch(&self) -> Result<TaskToManage, BotError> {
        let profile_id = "Test";
        Ok(TaskToManage::RemoveTasks(format!("{profile_id},")))
    }
}
