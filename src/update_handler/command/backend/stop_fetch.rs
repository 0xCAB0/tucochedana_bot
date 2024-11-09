use crate::{
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn stop_fetch(&mut self) -> Result<TaskToManage, BotError> {
        if !self.chat.active {
            self.api
                .send_message_without_reply(self.chat.id, "Las alertas ya han sido desactivadas")
                .await?;
            return Ok(TaskToManage::NoTask);
        }

        self.repo.modify_active_chat(&self.chat.id, false).await?;
        self.chat.active = false;

        let result = if let Some(active_subs) = &self.chat.subscribed_vehicles {
            //If he's the only subscriber, stop fetch
            Ok(TaskToManage::RemoveTasks(active_subs.to_owned()))
        } else {
            Ok(TaskToManage::NoTask)
        };

        self.start_message(
            "Alerta activada correctamente, le avisaremos si se registra alguno de sus vehículos",
        )
        .await?;

        result
    }
}
