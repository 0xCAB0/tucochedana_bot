use std::str::FromStr;

use bb8_postgres::{
    bb8::Pool,
    tokio_postgres::{NoTls, Row},
    PostgresConnectionManager,
};
use chrono::{DateTime, Utc};
use cron::Schedule;
use tokio::sync::OnceCell;

use crate::DATABASE_URL;

use super::{
    model::{chat::Chat, client_state::ClientState, vehicle::Vehicle},
    BotDbError,
};

static REPO: OnceCell<Repo> = OnceCell::const_new();

const INSERT_CHAT: &str = include_str!("queries/insert_chat.sql");
const INSERT_VEHICLE: &str = include_str!("queries/insert_vehicle.sql");
const DELETE_CHAT: &str = include_str!("queries/delete_chat.sql");
const CHECK_CHAT_EXISTS: &str = include_str!("queries/check_chat_exists.sql");
const GET_CHAT: &str = include_str!("queries/get_chat.sql");
const GET_VEHICLE: &str = include_str!("queries/get_vehicle.sql");
const GET_VEHICLES: &str = include_str!("queries/get_vehicles.sql");
const MODIFY_STATE: &str = include_str!("queries/modify_state.sql");
const CONCANT_CHAT_TO_SUBSCRIBERS: &str = include_str!("queries/concat_to_subscribers.sql");
const CONCAT_VEHICLE_TO_SUBSCRIPTIONS: &str =
    include_str!("queries/concat_to_subscribed_vehicles.sql");
const _DELETE_VEHICLE: &str = include_str!("queries/delete_vehicle.sql");
const _DELETE_ALL_FANG_TASKS_BY_PROFILE_ID: &str =
    include_str!("queries/delete_all_tasks_by_profile_id.sql");
const DELETE_FETCH_TASK_BY_CHAT_ID: &str =
    include_str!("queries/delete_fetch_tasks_by_profile_id.sql");

const UPDATE_SUBSCRIBED_CHATS: &str = include_str!("queries/update_subscribed_chats.sql");

pub struct Repo {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Repo {
    pub fn get_connection(&self) -> &Pool<PostgresConnectionManager<NoTls>> {
        &self.pool
    }

    pub async fn repo() -> Result<&'static Repo, BotDbError> {
        REPO.get_or_try_init(Repo::new).await
    }

    async fn pool(url: &str) -> Result<Pool<PostgresConnectionManager<NoTls>>, BotDbError> {
        let pg_mgr = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;

        Ok(Pool::builder().build(pg_mgr).await?)
    }

    pub async fn new() -> Result<Self, BotDbError> {
        let pl = Self::pool(&DATABASE_URL).await?;
        Ok(Repo { pool: pl })
    }

    #[cfg(test)]
    pub async fn new_no_tls() -> Result<Self, BotDbError> {
        use crate::DATABASE_URL;

        let pl = Self::pool(&DATABASE_URL).await?;
        Ok(Repo { pool: pl })
    }

    /* //
    Eventualmente se podrían usar para obtener el user_id que es un u64
    postgres_types no tiene compatibilidad con el tipo u64 , pero lo que hacemos en guardar el u64 como bytes en la BBDD de postgreSQL

    fn bytes_to_u64(bytes: &[u8]) -> u64 {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Self::as_u64_le(&arr)
    }*/

    pub fn as_u64_le(array: &[u8; 8]) -> u64 {
        (array[0] as u64)
            + ((array[1] as u64) << 8)
            + ((array[2] as u64) << 16)
            + ((array[3] as u64) << 24)
            + ((array[4] as u64) << 32)
            + ((array[5] as u64) << 40)
            + ((array[6] as u64) << 48)
            + ((array[7] as u64) << 56)
    }

    async fn check_user_exists(&self, chat_id: &i64) -> Result<bool, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection.execute(CHECK_CHAT_EXISTS, &[chat_id]).await?;
        Ok(n == 1)
    }

    /// Returns if it has been created
    pub async fn find_or_create_chat(
        &self,
        chat_id: &i64,
        user_id: u64,
        username: &str,
        language_code: &Option<String>,
    ) -> Result<(Chat, bool), BotDbError> {
        if self.check_user_exists(chat_id).await? {
            let chat = self.get_chat(chat_id).await?;

            Ok((chat, false))
        } else {
            let chat = self
                .insert_chat(chat_id, user_id, username, language_code)
                .await?;

            Ok((chat, false))
        }
    }

    pub async fn find_or_create_vehicle(&self, plate: &str) -> Result<Vehicle, BotDbError> {
        match self.get_vehicle(plate).await {
            Ok(row) => Ok(row),
            Err(_) => self.insert_vehicle(plate).await,
        }
    }

    pub fn calculate_next_delivery(cron_expression: &str) -> Result<DateTime<Utc>, BotDbError> {
        let schedule = Schedule::from_str(cron_expression)?;
        let mut iterator = schedule.upcoming(Utc);

        iterator.next().ok_or(BotDbError::NoTimestampsError)
    }

    pub async fn get_chat(&self, chat_id: &i64) -> Result<Chat, BotDbError> {
        let connection = self.pool.get().await?;

        let row = match connection.query_one(GET_CHAT, &[chat_id]).await {
            Ok(r) => r,
            Err(err) => {
                log::error!("get_chat -> {}", err);
                return Err(BotDbError::PgError(err));
            }
        };

        Ok(row.into())
    }

    pub async fn get_rows(&self, query: String) -> Result<Vec<Row>, BotDbError> {
        let connection = self.pool.get().await?;

        Ok(connection.query(&query, &[]).await?)
    }

    pub async fn get_vehicles(
        &self,
        subscribed_vehicles: &String,
    ) -> Result<Vec<Vehicle>, BotDbError> {
        let connection = self.pool.get().await?;

        let rows = connection
            .query(GET_VEHICLES, &[subscribed_vehicles])
            .await?;

        let vehicles: Vec<Vehicle> = rows.into_iter().map(|row| row.into()).collect();

        Ok(vehicles)
    }

    pub async fn get_vehicle(&self, plate: &str) -> Result<Vehicle, BotDbError> {
        let connection = self.pool.get().await?;

        let row = connection.query_one(GET_VEHICLE, &[&plate]).await?;

        Ok(row.into())
    }

    pub async fn subscribe_chat_id_to_vehicle(
        &self,
        plate: &str,
        chat_id: i64,
    ) -> Result<(), BotDbError> {
        let current_subscriptions = self.get_subscriptions_from_vehicle_as_string(plate).await?;

        if current_subscriptions.is_some_and(|list| {
            list.split(',')
                .any(|subscribed_id| subscribed_id == chat_id.to_string())
        }) {
            return Err(BotDbError::AlreadySubscribedError(
                chat_id,
                plate.to_string(),
            ));
        }

        let mut connection = self.pool.get().await?;

        let transaction = connection.transaction().await?;

        let n1 = transaction
            .execute(CONCANT_CHAT_TO_SUBSCRIBERS, &[&chat_id.to_string(), &plate])
            .await?;
        let n2 = transaction
            .execute(CONCAT_VEHICLE_TO_SUBSCRIPTIONS, &[&chat_id, &plate])
            .await?;

        if n1 == n2 {
            transaction.commit().await?;
            Ok(())
        } else {
            log::error!("{n1} != {n2}");
            transaction.rollback().await?;
            Err(BotDbError::AlreadySubscribedError(
                chat_id,
                plate.to_string(),
            ))
        }
    }

    pub async fn unsubscribe_chat_id_to_vehicle(
        &self,
        plate: &str,
        chat_id: i64,
    ) -> Result<u64, BotDbError> {
        let current_subscriptions = self.get_subscriptions_from_vehicle_as_string(plate).await?;

        if current_subscriptions.is_none() {
            return Ok(0);
        }
        let updated_subscriptions = current_subscriptions
            .unwrap()
            .split(",")
            .filter(|x| *x == chat_id.to_string())
            .collect::<String>();

        let connection = self.pool.get().await?;

        let n = connection
            .execute(UPDATE_SUBSCRIBED_CHATS, &[&plate, &updated_subscriptions])
            .await?;

        Ok(n)
    }

    pub async fn get_subscriptions_from_vehicle(
        &self,
        plate: &str,
    ) -> Result<Vec<i64>, BotDbError> {
        let chat_ids = self.get_vehicle(plate).await?.subscribers_ids;

        let result = if chat_ids.is_none() {
            vec![]
        } else {
            let chat_ids_str = chat_ids.unwrap();
            let mut split = chat_ids_str.split(',');

            split.next_back(); // Removes the empty item
            split
                .filter_map(|id| id.parse::<i64>().ok())
                .collect::<Vec<i64>>()
        };

        Ok(result)
    }

    pub async fn get_subscriptions_from_vehicle_as_string(
        &self,
        plate: &str,
    ) -> Result<Option<String>, BotDbError> {
        Ok(self.get_vehicle(plate).await?.subscribers_ids)
    }

    async fn insert_chat(
        &self,
        chat_id: &i64,
        user_id: u64,
        username: &'_ str,
        language_code: &Option<String>,
    ) -> Result<Chat, BotDbError> {
        let connection = self.pool.get().await?;

        let bytes = user_id.to_le_bytes().to_vec();

        let row = match connection
            .query_one(
                INSERT_CHAT,
                &[
                    chat_id,
                    &ClientState::Initial,
                    &bytes,
                    &username,
                    language_code,
                ],
            )
            .await
        {
            Ok(r) => r,
            Err(err) => {
                log::error!("insert_chat -> {}", err);
                return Err(BotDbError::PgError(err));
            }
        };
        Ok(row.into())
    }

    pub async fn insert_vehicle(&self, plate: &str) -> Result<Vehicle, BotDbError> {
        let connection = self.pool.get().await?;

        let row = match connection.query_one(INSERT_VEHICLE, &[&plate]).await {
            Ok(r) => r.into(),
            Err(err) => {
                log::error!("insert_chat -> {}", err);
                return Err(BotDbError::PgError(err));
            }
        };

        Ok(row)
    }

    pub async fn delete_chat(&self, chat_id: &i64) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection.execute(DELETE_CHAT, &[chat_id]).await?;
        Ok(n)
    }

    pub async fn modify_state(
        &self,
        chat_id: &i64,
        new_state: ClientState,
    ) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection
            .execute(MODIFY_STATE, &[&new_state, chat_id])
            .await?;
        Ok(n)
    }

    pub async fn delete_tasks_by_chat_id(&self, chat_id: &'_ str) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;
        let n = connection
            .execute(DELETE_FETCH_TASK_BY_CHAT_ID, &[&chat_id])
            .await?;
        Ok(n)
    }
}

#[cfg(test)]
mod db_tests {
    use std::ops::Not;

    use super::*;

    async fn clear_database() -> Result<(), BotDbError> {
        dotenvy::dotenv().ok();
        let db_controller = Repo::new_no_tls().await.unwrap();
        let connection = db_controller.get_connection().get().await?;

        let _ = &connection.execute("DELETE FROM chats", &[]).await?;
        let _ = &connection.execute("DELETE FROM vehicles", &[]).await?;
        Ok(())
    }

    async fn populate_database() -> Result<(), BotDbError> {
        dotenvy::dotenv().ok();
        let db_controller = Repo::new_no_tls().await.unwrap();
        let connection = db_controller.get_connection().get().await?;

        // Insert 3 test users into the `chats` table using connection.execute
        connection
            .execute(
                "INSERT INTO chats (id, user_id, username, language_code, subscribed_vehicles) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &1_i64,
                    &123456_u64.to_le_bytes().to_vec(),
                    &"user1",
                    &Some("en".to_string()),
                    &"ABC123,DEF456,"
                ],
            )
            .await?;
        connection
            .execute(
                "INSERT INTO chats (id, user_id, username, language_code, subscribed_vehicles) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &2_i64,
                    &234567_u64.to_le_bytes().to_vec(),
                    &"user2",
                    &Some("fr".to_string()),
                    &"DEF456,"
                ],
            )
            .await?;
        connection
            .execute(
                "INSERT INTO chats (id, user_id, username, language_code) VALUES ($1, $2, $3, $4)",
                &[
                    &3_i64,
                    &345678_u64.to_le_bytes().to_vec(),
                    &"user3",
                    &Some("es".to_string()),
                ],
            )
            .await?;

        // Insert 3 test vehicles into the `vehicles` table using connection.execute
        connection
            .execute(
                "INSERT INTO vehicles (plate, subscribers_ids) VALUES ($1, $2)",
                &[&"ABC123", &"1,"],
            )
            .await?;
        connection
            .execute(
                "INSERT INTO vehicles (plate, subscribers_ids) VALUES ($1, $2)",
                &[&"DEF456", &"1,2,"],
            )
            .await?;
        connection
            .execute("INSERT INTO vehicles (plate) VALUES ($1)", &[&"GHI789"])
            .await?;

        Ok(())
    }
    #[tokio::test]
    async fn test_modify_state() {
        clear_database().await.unwrap();
        //Pick a random user of the DB
        let db_controller = Repo::new_no_tls().await.unwrap();
        let connection = db_controller.get_connection().get().await.unwrap();

        let chat = db_controller
            .insert_chat(&999, 1111111, "hello", &Some("en".to_string()))
            .await
            .unwrap();

        assert_eq!(chat.id, 999_i64);
        assert_eq!(chat.user_id, 1111111_u64);

        let row: &Row = &connection
            .query_one("SELECT * FROM chats LIMIT 1", &[])
            .await
            .unwrap();

        let chat_id: i64 = row.get("id");
        //testing modify state

        let n = db_controller
            .modify_state(&chat_id, ClientState::AddVehicle)
            .await
            .unwrap();

        assert_eq!(n, 1_u64);

        //testing get state
        let chat = db_controller.get_chat(&chat_id).await.unwrap();

        assert_eq!(chat.state, ClientState::AddVehicle);

        let n = db_controller
            .modify_state(&chat_id, ClientState::Initial)
            .await
            .unwrap();
        assert_eq!(n, 1_u64);

        let chat = db_controller.get_chat(&chat_id).await.unwrap();

        assert_eq!(chat.state, ClientState::Initial);

        let n = db_controller.delete_chat(&999).await.unwrap();
        assert_eq!(n, 1_u64);
    }

    #[tokio::test]
    async fn test_subscribe_to_vehicle() {
        clear_database().await.unwrap();
        populate_database().await.unwrap();

        let db_controller = Repo::new_no_tls().await.unwrap();
        let _connection = db_controller.get_connection().get().await.unwrap();

        let testing_plate = "GHI789";
        let testing_chat = 3;

        // Añadir a uno vacío
        db_controller
            .subscribe_chat_id_to_vehicle(testing_plate, testing_chat)
            .await
            .unwrap();

        let chat = db_controller.get_chat(&3).await.unwrap();

        assert!(chat
            .subscribed_vehicles
            .clone()
            .is_some_and(|t| t.is_empty().not()));

        let vehicle = db_controller.get_vehicle(testing_plate).await.unwrap();

        assert!(&chat
            .subscribed_vehicles
            .clone()
            .unwrap()
            .split(',')
            .any(|subbed| subbed == testing_plate));

        assert!(&vehicle
            .subscribers_ids
            .clone()
            .unwrap()
            .split(',')
            .any(|subscriber| subscriber == testing_chat.to_string()))
        // Añadir a uno con contenido

        //db_controller.subscribe_chat_id_to_vehicle(plate, chat_id)
    }
    // #[tokio::test]
    // async fn list_my_vehicles() {
    //     clear_database().await.unwrap();
    //     populate_database().await.unwrap();
    // }
}
