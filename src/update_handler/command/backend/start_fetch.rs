use crate::{
    tasks::fetch::FetchTask,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn start_fetch(&self) -> Result<TaskToManage, BotError> {
        //Set chat as active
        self.repo.modify_active_chat(&self.chat.id, true).await?;

        let Some(subbs) = &self.chat.subscribed_vehicles else {
            self.api
                .send_message_without_reply(
                    self.chat.id,
                    "Debe añadir vehículos para activar las alertas".to_string(),
                )
                .await?;
            return Ok(TaskToManage::NoTask);
        };

        let mut tasks = vec![];
        //For all subscribed vehicles, try create a fetch Task

        for sub in subbs.split(',') {
            let task = FetchTask::builder().plate(sub.to_string()).build();
            tasks.push(task);
        }

        Ok(TaskToManage::FetchTasks(tasks))
    }
}
