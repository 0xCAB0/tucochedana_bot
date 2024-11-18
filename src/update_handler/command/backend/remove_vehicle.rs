use crate::{
    db::BotDbError,
    update_handler::process_update::{TaskToManage, UpdateProcessor},
    BotError,
};

impl UpdateProcessor {
    pub async fn remove_vehicle(&self) -> Result<TaskToManage, BotError> {
        let mut iter = self.get_parse_iterator();
        let plate: &str = iter.next().unwrap();

        // 1. Remove it from chat.subscribed_vehicles
        // 2. Remove chat from vehicle subscribers_ids
        match self.repo.end_subscription(plate, self.chat.id).await {
            Ok((n_subscribers, n_subscriptions)) => {
                if n_subscriptions == 0 {
                    self.repo.modify_active_chat(&self.chat.id, false).await?;
                    self.start_message(Some(&format!(
                        "El vehículo {plate} ha sido eliminado correctamente✅\n
                    Hemos desactivado las alertas pues no tiene ningún otro coche añadido"
                    )))
                    .await?;
                } else {
                    self.get_vehicles(Some(&format!(
                        "El vehículo {plate} ha sido eliminado correctamente✅"
                    )))
                    .await?;
                };
                // 3. If vehicle subscribers_ids is empty -> Remove task & delete vehicle
                if n_subscribers == 0 {
                    Ok(TaskToManage::RemoveTask(String::from(plate)))
                } else {
                    Ok(TaskToManage::NoTask)
                }
            }

            Err(BotDbError::CouldNotEndSubscription(_, _, reason)) => {
                self.get_vehicles(Some(&reason)).await?;
                Ok(TaskToManage::NoTask)
            }
            Err(err) => Err(BotError::DbError(err)),
        }
    }
}
