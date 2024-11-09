use crate::{
    tasks::fetch::FetchTask,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn start_fetch(&mut self) -> Result<TaskToManage, BotError> {
        if self.chat.active {
            self.api
                .send_message_without_reply(self.chat.id, "Las alertas ya han sido activadas")
                .await?;
            return Ok(TaskToManage::NoTask);
        }

        self.repo.modify_active_chat(&self.chat.id, true).await?;

        self.chat.active = true;

        let Some(subbs) = &self.chat.subscribed_vehicles else {
            self.api
                .send_message_without_reply(
                    self.chat.id,
                    "Debe añadir vehículos para activar las alertas",
                )
                .await?;
            return Ok(TaskToManage::NoTask);
        };

        let mut tasks = vec![];

        for sub_vehicles in subbs.split(',') {
            let task = FetchTask::builder().plate(sub_vehicles.to_string()).build();
            tasks.push(task);
        }

        self.start_message(
            Some("Alerta activada correctamente ✅, le avisaremos si se registra alguno de sus vehículos"),
        )
        .await?;

        Ok(TaskToManage::FetchTasks(tasks))
    }
}
