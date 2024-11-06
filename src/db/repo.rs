use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use tokio::sync::OnceCell;

use crate::DATABASE_URL;

use super::{
    model::{chat::Chat, client_state::ClientState},
    BotDbError,
};

static REPO: OnceCell<Repo> = OnceCell::const_new();

const INSERT_CHAT: &str = include_str!("queries/insert_chat.sql");
const DELETE_CHAT: &str = include_str!("queries/delete_chat.sql");
const GET_CHAT: &str = include_str!("queries/get_chat.sql");
const GET_VEHICLES_BY_CHAT_ID: &str = include_str!("queries/get_vehicles_by_chat_id.sql");
const MODIFY_STATE: &str = include_str!("queries/modify_state.sql");
const MODIFY_ACTIVE: &str = include_str!("queries/modify_active.sql");
const CONCAT_TO_SELECTED_PROFILES: &str = include_str!("queries/concat_to_selected_profiles.sql");
const DELETE_VEHICLE: &str = include_str!("queries/delete_vehicle.sql");
const DELETE_TASK_BY_PROFILE_ID: &str = include_str!("queries/delete_all_tasks_by_profile_id.sql");
const DELETE_FETCH_TASK_BY_PROFILE_ID: &str =
    include_str!("queries/delete_fetch_tasks_by_profile_id.sql");

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

    pub async fn check_user_exists(&self, chat_id: &i64) -> Result<bool, super::BotDbError> {
        todo!()
    }

    pub async fn find_or_create_chat(
        &self,
        chat_id: &i64,
        user_id: u64,
        username: &str,
        language_code: &Option<String>,
    ) -> Result<Chat, super::BotDbError> {
        todo!()
    }

    pub fn calculate_next_delivery(
        cron_expression: &str,
    ) -> Result<chrono::DateTime<chrono::Utc>, super::BotDbError> {
        todo!()
    }

    pub async fn get_chat(&self, chat_id: &i64) -> Result<Chat, super::BotDbError> {
        todo!()
    }

    pub async fn get_rows(
        &self,
        query: String,
    ) -> Result<Vec<bb8_postgres::tokio_postgres::Row>, super::BotDbError> {
        todo!()
    }

    pub async fn insert_chat(
        &self,
        chat_id: &i64,
        user_id: u64,
        username: &'_ str,
        language_code: &Option<String>,
    ) -> Result<Chat, super::BotDbError> {
        todo!()
    }

    pub async fn delete_chat(&self, chat_id: &i64) -> Result<u64, super::BotDbError> {
        todo!()
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
            .execute(DELETE_TASK_BY_PROFILE_ID, &[&chat_id])
            .await?;
        Ok(n)
    }
}
