use crate::{
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn stop_fetch(&self) -> Result<TaskToManage, BotError> {
        if !self.chat.active {
            self.api
                .send_message_without_reply(
                    self.chat.id,
                    "Las alertas ya han sido desactivadas".to_string(),
                )
                .await?;
            return Ok(TaskToManage::NoTask);
        }

        self.repo.modify_active_chat(&self.chat.id, false).await?;

        if let Some(active_subs) = &self.chat.subscribed_vehicles {
            //If he's the only subscriber, stop fetch
            Ok(TaskToManage::RemoveTasks(active_subs.to_owned()))
        } else {
            Ok(TaskToManage::NoTask)
        }
    }
}
