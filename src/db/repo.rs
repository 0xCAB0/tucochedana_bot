use bb8_postgres::{bb8::Pool, tokio_postgres::NoTls, PostgresConnectionManager};
use tokio::sync::OnceCell;

use crate::DATABASE_URL;

use super::{model::chat::Chat, BotDbError};

static REPO: OnceCell<Repo> = OnceCell::const_new();

pub struct Repo{
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Repo {
    pub fn get_connection(&self) -> &Pool<PostgresConnectionManager<NoTls>> {
        &self.pool
    }

    pub async fn repo() -> Result<&'static Repo, BotDbError> {
        REPO.get_or_try_init(Repo::new).await
    }

    async fn pool(
        url: &str,
    ) -> Result<Pool<PostgresConnectionManager<NoTls>>, BotDbError> {

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

    pub fn calculate_next_delivery(cron_expression: &str) -> Result<chrono::DateTime<chrono::Utc>, super::BotDbError> {
        todo!()
    }

    pub async fn get_user(&self, chat_id: &i64) -> Result<Chat, super::BotDbError> {
    todo!()
    }

    pub async fn get_rows(&self, query: String) -> Result<Vec<bb8_postgres::tokio_postgres::Row>, super::BotDbError> {
        todo!()
    }

    pub async fn insert_user(
        &self,
        chat_id: &i64,
        user_id: u64,
        username: &'_ str,
        language_code: &Option<String>,
    ) -> Result<Chat, super::BotDbError> {
        todo!()
    }

    pub async fn insert_profile(&self, chat_id: &i64) -> Result<i32, super::BotDbError> {
        todo!()
    }

    pub async fn delete_chat(&self, chat_id: &i64) -> Result<u64, super::BotDbError> {
        todo!()
    }

    pub async fn modify_profile_name(
        &self,
        profile_id: &i32,
        profile_name: &'_ str,
    ) -> Result<u64, super::BotDbError> {
        todo!()
    }

    pub async fn modify_offset(&self, chat_id: &i64, offset: i8) -> Result<u64, super::BotDbError> {
        todo!()
    }

    pub async fn modify_selected(&self, chat_id: &i64, new_selected: String)
        -> Result<u64, super::BotDbError> {
        todo!()
    }

    pub async fn modify_selected_profiles(
        &self,
        chat_id: &i64,
        profiles: Option<&str>,
    ) -> Result<u64, super::BotDbError> {
        todo!()
    }

    pub async fn concat_to_selected_profiles(
        &self,
        chat_id: &i64,
        profiles: &str,
    ) -> Result<u64, super::BotDbError> {
        todo!()
    }
}