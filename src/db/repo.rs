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
const INSERT_VEHICLE_PLATE: &str = include_str!("queries/insert_vehicle_plate.sql");
const DELETE_CHAT: &str = include_str!("queries/delete_chat.sql");
const CHECK_CHAT_EXISTS: &str = include_str!("queries/check_chat_exists.sql");
const GET_CHAT: &str = include_str!("queries/get_chat.sql");
const GET_VEHICLE: &str = include_str!("queries/get_vehicle.sql");
const GET_VEHICLES: &str = include_str!("queries/get_vehicles.sql");
const MODIFY_STATE: &str = include_str!("queries/modify_state.sql");
const MODIFY_ACTIVE_CHAT: &str = include_str!("queries/modify_active_chat.sql");
const MODIFY_FOUND_AT_VEHICLE: &str = include_str!("queries/modify_found_at vehicle.sql");
const CONCANT_CHAT_TO_SUBSCRIBERS: &str = include_str!("queries/concat_to_subscribers.sql");
const CONCAT_VEHICLE_TO_SUBSCRIPTIONS: &str =
    include_str!("queries/concat_to_subscribed_vehicles.sql");
const _DELETE_VEHICLE: &str = include_str!("queries/delete_vehicle.sql");
const _DELETE_ALL_FANG_TASKS_BY_PROFILE_ID: &str =
    include_str!("queries/delete_all_tasks_by_profile_id.sql");
const DELETE_FETCH_TASK_BY_PLATE: &str = include_str!("queries/delete_fetch_tasks_by_plate.sql");
const MODIFY_SUBSCRIBERS_VEHICLE: &str = include_str!("queries/modify_subscribers_vehicle.sql");
const MODIFY_SUBSCRIBED_CHAT: &str = include_str!("queries/modify_subscribed_chats.sql");
const FILTER_ACTIVE_CHATS: &str = include_str!("queries/filter_active_chats.sql");
const COUNT_SUBSCRIBERS_PLATE: &str = include_str!("queries/count_subscribers_plate.sql");

pub struct Repo {
    pub(crate) pool: Pool<PostgresConnectionManager<NoTls>>,
    #[cfg(test)]
    pub(crate) database_name: Option<String>,
}

/// Setup methods
impl Repo {
    pub fn get_connection(&self) -> &Pool<PostgresConnectionManager<NoTls>> {
        &self.pool
    }

    pub async fn repo() -> Result<&'static Repo, BotDbError> {
        REPO.get_or_try_init(Repo::new).await
    }

    pub async fn pool(url: &str) -> Result<Pool<PostgresConnectionManager<NoTls>>, BotDbError> {
        let pg_mgr = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;

        Ok(Pool::builder().build(pg_mgr).await?)
    }

    pub async fn new() -> Result<Self, BotDbError> {
        let pl = Self::pool(&DATABASE_URL).await?;
        Ok(Repo {
            pool: pl,
            #[cfg(test)]
            database_name: None,
        })
    }

    #[cfg(test)]
    pub async fn new_no_tls() -> Result<Self, BotDbError> {
        use crate::DATABASE_URL;

        let pl = Self::pool(&DATABASE_URL).await?;
        Ok(Repo {
            pool: pl,
            database_name: None,
        })
    }

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
}

/// Queries
impl Repo {
    // General methods
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

    // Getters
    pub async fn get_vehicles_by_chat_id(&self, chat_id: &i64) -> Result<Vec<Vehicle>, BotDbError> {
        let chat = self.get_chat(chat_id).await?;

        match chat.subscribed_vehicles {
            Some(mut subs) => self.get_vehicles_from_subs_string(&mut subs).await,

            None => Ok(vec![]),
        }
    }

    async fn get_vehicles_from_subs_string(
        &self,
        subscribed_vehicles: &mut String,
    ) -> Result<Vec<Vehicle>, BotDbError> {
        let connection = self.pool.get().await?;

        subscribed_vehicles.retain(|c| !c.is_whitespace());

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

    pub async fn get_active_subscriptions_from_vehicle(
        &self,
        plate: &str,
    ) -> Result<Vec<Chat>, BotDbError> {
        let chat_ids = self.get_vehicle(plate).await?.subscribers_ids;

        let result = if chat_ids.is_none() {
            vec![]
        } else {
            let connection = self.pool.get().await?;

            let mut chat_ids_str = chat_ids.unwrap();
            chat_ids_str.pop(); // Removes the last ","
            chat_ids_str.retain(|c| !c.is_whitespace());

            let active_chats: Vec<Row> = connection
                .query(FILTER_ACTIVE_CHATS, &[&chat_ids_str])
                .await?;
            active_chats.into_iter().map(|row| row.into()).collect()
        };

        Ok(result)
    }

    pub async fn get_subscriptions_from_vehicle_as_string(
        &self,
        plate: &str,
    ) -> Result<Option<String>, BotDbError> {
        Ok(self.get_vehicle(plate).await?.subscribers_ids)
    }

    async fn check_user_exists(&self, chat_id: &i64) -> Result<bool, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection.execute(CHECK_CHAT_EXISTS, &[chat_id]).await?;
        Ok(n == 1)
    }

    // Inserts

    /// Returns true if insert, false if fetched
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

            Ok((chat, true))
        }
    }

    pub async fn find_or_create_vehicle(&self, plate: &str) -> Result<Vehicle, BotDbError> {
        match self.get_vehicle(plate).await {
            Ok(row) => Ok(row),
            Err(_) => self.insert_vehicle_plate(plate).await,
        }
    }

    pub async fn insert_vehicle(&self, vehicle: Vehicle) -> Result<Vehicle, BotDbError> {
        let connection = self.pool.get().await?;

        let row = match connection
            .query_one(
                INSERT_VEHICLE,
                &[&vehicle.plate, &vehicle.subscribers_ids, &vehicle.found_at],
            )
            .await
        {
            Ok(r) => r.into(),
            Err(err) => {
                log::error!("insert_vehicle -> {}", err);
                return Err(BotDbError::PgError(err));
            }
        };

        Ok(row)
    }

    async fn insert_vehicle_plate(&self, plate: &str) -> Result<Vehicle, BotDbError> {
        let connection = self.pool.get().await?;

        let row = match connection.query_one(INSERT_VEHICLE_PLATE, &[&plate]).await {
            Ok(r) => r.into(),
            Err(err) => {
                log::error!("insert_vehicle_by_plate -> {}", err);
                return Err(BotDbError::PgError(err));
            }
        };

        Ok(row)
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

    // Deletes
    pub async fn delete_chat(&self, chat_id: &i64) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection.execute(DELETE_CHAT, &[chat_id]).await?;
        Ok(n)
    }

    // Update
    pub async fn modify_found_at_vehicle(
        &self,
        plate: &str,
        found_at: DateTime<Utc>,
    ) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection
            .execute(MODIFY_FOUND_AT_VEHICLE, &[&found_at, &plate])
            .await?;
        Ok(n)
    }

    pub async fn modify_active_chat(
        &self,
        chat_id: &i64,
        new_state: bool,
    ) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;

        let n = connection
            .execute(MODIFY_ACTIVE_CHAT, &[&new_state, chat_id])
            .await?;
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

    //Subscriptions

    pub async fn get_n_subscribers_by_plate(&self, plate: &str) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;
        let n = connection
            .execute(COUNT_SUBSCRIBERS_PLATE, &[&plate])
            .await?;
        Ok(n)
    }

    pub async fn append_subscription_to_chat(
        &self,
        plate: &str,
        chat_id: &i64,
    ) -> Result<(), BotDbError> {
        let connection = self.pool.get().await?;

        match connection
            .execute(CONCAT_VEHICLE_TO_SUBSCRIPTIONS, &[&chat_id, &plate])
            .await
        {
            Ok(n) if n > 0 => Ok(()),
            Ok(_) => Err(BotDbError::AlreadySubscribedError(
                *chat_id,
                plate.to_string(),
            )),
            Err(err) => Err(BotDbError::PgError(err)),
        }
    }

    pub async fn create_subscription(&self, plate: &str, chat_id: i64) -> Result<(), BotDbError> {
        let current_subscriptions = self.get_subscriptions_from_vehicle_as_string(plate).await?;

        if current_subscriptions.is_some_and(|list| {
            list.split(',')
                .map(str::trim)
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
            .execute(CONCAT_VEHICLE_TO_SUBSCRIPTIONS, &[&plate, &chat_id])
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

    /// Returns the new size of subscriptions and subscribers lists
    pub async fn end_subscription(
        &self,
        plate: &str,
        chat_id: i64,
    ) -> Result<(u64, u64), BotDbError> {
        let current_subscribers = self.get_subscriptions_from_vehicle_as_string(plate).await?;
        let current_subscriptions = self.get_chat(&chat_id).await?.subscribed_vehicles;

        let (n_subscribers, updated_subscribers) =
            Self::pop_member_from_subs_string(current_subscribers, chat_id.to_string());

        let (n_subscriptions, updated_subscriptions) =
            Self::pop_member_from_subs_string(current_subscriptions, plate.to_string());

        if n_subscribers == 0 || n_subscriptions == 0 {
            let reason = if n_subscribers == 0 {
                format!("The vehicle {plate} doesn't have any subscribers")
            } else {
                format!("User {chat_id} doesn't have any subscription at the moment")
            };
            return Err(BotDbError::CouldNotEndSubscription(
                chat_id,
                plate.to_string(),
                reason,
            ));
        }

        let mut connection = self.pool.get().await?;
        let transaction = connection.transaction().await?;

        let n = transaction
            .execute(MODIFY_SUBSCRIBERS_VEHICLE, &[&updated_subscribers, &plate])
            .await?;

        let n1 = transaction
            .execute(MODIFY_SUBSCRIBED_CHAT, &[&updated_subscriptions, &chat_id])
            .await?;

        if n != n1 {
            transaction.rollback().await?;
            return Err(BotDbError::CouldNotEndSubscription(
                chat_id,
                plate.to_string(),
                "Update query failed".to_string(),
            ));
        }

        transaction.commit().await?;

        Ok((n_subscribers, n_subscriptions))
    }

    /// Works for both chat.subscribed_vehicles and vehicle.subscribed_ids
    fn pop_member_from_subs_string(
        subscription_string: Option<String>,
        member: String,
    ) -> (u64, String) {
        match subscription_string {
            Some(subscribers) => {
                let valid_items: Vec<_> = subscribers
                    .split(',')
                    .map(str::trim)
                    .filter(|&x| x == member)
                    .collect();

                (valid_items.len() as u64, valid_items.join(","))
            }
            None => (0, String::new()),
        }
    }

    pub async fn delete_tasks_by_plate(&self, plate: &str) -> Result<u64, BotDbError> {
        let connection = self.pool.get().await?;
        let n = connection
            .execute(DELETE_FETCH_TASK_BY_PLATE, &[&plate])
            .await?;
        Ok(n)
    }
}

#[cfg(test)]
mod db_tests {
    use chrono::{Duration, Timelike};
    use diesel::{Connection, PgConnection, RunQueryDsl};
    use diesel_migrations::{FileBasedMigrations, MigrationHarness};
    use fang::{AsyncQueue, FangError};
    use rand::Rng;

    use std::ops::Not;

    use super::*;

    fn random_datetime() -> DateTime<Utc> {
        use chrono::Utc;

        let now = Utc::now();
        let mut rng = rand::thread_rng();

        // Generate a random number of days to add/subtract (e.g., -365 to +365)
        let days = rng.gen_range(-365..365);
        // Generate random hours and minutes
        let hours = rng.gen_range(0..24);
        let minutes = rng.gen_range(0..60);

        // Adjust the datetime by the random amounts
        let random_time =
            now + Duration::days(days) + Duration::hours(hours) + Duration::minutes(minutes);
        random_time
            .with_nanosecond((random_time.nanosecond() / 1_000) * 1_000)
            .unwrap()
    }

    fn construct_test_db_url(base_url: &str, test_db_name: &str) -> String {
        // Split the URL at the last `/` to get the base connection string without the original database name
        let pos = base_url.rfind('/').expect("Invalid database URL format");

        // Construct the new test database URL by replacing the original database name
        format!("{}{}", &base_url[..pos + 1], test_db_name)
    }

    impl Repo {
        #[cfg(test)]
        /// Sets up a new, unique test database for each test, applies migrations, and returns a `Repo` instance.
        pub async fn new_for_test(db_name: &str) -> Result<Self, BotDbError> {
            // Load environment variables from `.env`

            dotenvy::dotenv().ok();

            // Generate a unique test database name
            let test_db_name = format!("test_db_{}_{}", db_name, rand::random::<u32>());

            let mut connection = PgConnection::establish(&DATABASE_URL)
                .unwrap_or_else(|_| panic!("Error connecting to {:?}", *DATABASE_URL));

            let _ = diesel::sql_query(format!("CREATE DATABASE {test_db_name}"))
                .execute(&mut connection);

            // Construct the URL for the test database
            let test_db_url = construct_test_db_url(&DATABASE_URL, &test_db_name);

            let mut test_connection = PgConnection::establish(&test_db_url)
                .unwrap_or_else(|_| panic!("Error connecting to {:?}", test_db_url));

            // Paths to migrations directories
            let project_root = std::env::current_dir().expect("Could not get project root");
            let migrations_dir = FileBasedMigrations::find_migrations_directory()
                .expect("Could not find 'migrations' directory");
            let testing_migrations_dir =
                FileBasedMigrations::from_path(project_root.join("testing-migrations"))
                    .expect("Could not find testing migrations dir");

            // Run migrations in the `migrations` directory
            test_connection
                .run_pending_migrations(migrations_dir)
                .expect("Error running migrations from 'migrations'");

            // Run migrations in the `testing-migrations` directory
            test_connection
                .run_pending_migrations(testing_migrations_dir)
                .expect("Error running migrations from 'testing-migrations'");

            let pool = Repo::pool(&test_db_url).await?;
            Ok(Repo {
                pool,
                database_name: Some(test_db_name),
            })
        }

        /// Cleans up the test database by dropping it.
        pub async fn cleanup_test_db(&self) -> Result<(), BotDbError> {
            drop(self.get_connection().get().await?);
            let admin_repo = Repo::new_no_tls().await?;
            let connection = admin_repo.get_connection().get().await.unwrap();

            let db_name = match self.database_name.clone() {
                Some(name) => name,
                None => return Ok(()), // Return Ok(()) if the value is None
            };

            // Terminate all active connections to the target database
            let terminate_connections_query = format!(
                "SELECT pg_terminate_backend(pg_stat_activity.pid) \
         FROM pg_stat_activity \
         WHERE pg_stat_activity.datname = '{}' \
         AND pid <> pg_backend_pid();",
                db_name
            );

            connection
                .execute(&terminate_connections_query, &[])
                .await?;
            // Dynamically format the query to include the database name directly
            let drop_db_query = format!("DROP DATABASE IF EXISTS {}", db_name);

            connection.execute(&drop_db_query, &[]).await?;
            Ok(())
        }

        pub async fn create_testing_queue(
            &self,
            use_main: bool,
        ) -> Result<AsyncQueue<NoTls>, FangError> {
            use crate::DATABASE_URL;

            let url = if use_main {
                DATABASE_URL.to_string()
            } else {
                construct_test_db_url(&DATABASE_URL, &self.database_name.clone().unwrap())
            };

            let mut queue: AsyncQueue<NoTls> =
                AsyncQueue::builder().uri(url).max_pool_size(5_u32).build();

            queue.connect(NoTls).await.unwrap();
            Ok(queue)
        }
    }

    #[test]
    fn test_get_user_id() {
        let user_1 = 12356_u64;
        let user_2 = 12388456_u64;
        let user_3 = 1235996_u64;

        let bytes_1 = user_1.to_le_bytes().to_vec();
        let bytes_2 = user_2.to_le_bytes().to_vec();
        let bytes_3 = user_3.to_le_bytes().to_vec();

        println!("{:?}", bytes_1);
        println!("{:?}", bytes_2);
        println!("{:?}", bytes_3);
    }
    #[tokio::test]
    async fn test_modify_state() {
        //Pick a random user of the DB
        let db_controller = Repo::new_for_test("test_modify_state").await.unwrap();

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
        db_controller.cleanup_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_to_first_empty_vehicle() {
        let testing_plate = "GHI789";
        let testing_chat = 3;

        let db_controller = Repo::new_for_test("test_subscribe_to_first_empty_vehicle")
            .await
            .unwrap();

        let (chat, vehicle) =
            test_subscribe_to_vehicle(testing_plate, testing_chat, &db_controller)
                .await
                .unwrap();

        let subscribers: Vec<i64> = vehicle
            .subscribers_ids
            .unwrap()
            .split(',')
            .filter_map(|x| {
                match x.parse::<i64>() {
                    Ok(value) => Some(value), // Keep successfully parsed values
                    Err(e) => {
                        println!("Failed to parse '{}': {}", x, e); // Log the error
                        None // Skip the value if parsing fails
                    }
                }
            })
            .collect();

        let subscriptions: Vec<String> = chat
            .subscribed_vehicles
            .unwrap()
            .split(',')
            .map(|x| x.to_string())
            .collect();

        assert!(
            subscribers.len() == 1,
            "subscribers -> {}",
            subscribers.len()
        );
        assert!(
            subscriptions.len() == 1,
            "subscriptions -> {}",
            subscriptions.len()
        );

        db_controller.cleanup_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_to_first_vehicle() {
        let testing_plate = "DEF456";
        let testing_chat = 3;

        let db_controller = Repo::new_for_test("test_subscribe_to_first_vehicle")
            .await
            .unwrap();

        let (chat, vehicle) =
            test_subscribe_to_vehicle(testing_plate, testing_chat, &db_controller)
                .await
                .unwrap();

        let subscribers: Vec<i64> = vehicle
            .subscribers_ids
            .unwrap()
            .split(',')
            .filter_map(|x| {
                match x.parse::<i64>() {
                    Ok(value) => Some(value), // Keep successfully parsed values
                    Err(_) => None,           // Skip the value if parsing fails
                }
            })
            .collect();

        let subscriptions: Vec<String> = chat
            .subscribed_vehicles
            .unwrap()
            .split(',')
            .map(|x| x.to_string())
            .collect();

        assert!(
            subscribers.len() == 3,
            "subscribers -> {}",
            subscribers.len()
        );
        assert!(
            subscriptions.len() == 1,
            "subscriptions -> {}",
            subscriptions.len()
        );

        db_controller.cleanup_test_db().await.unwrap();
    }

    async fn test_subscribe_to_vehicle(
        testing_plate: &str,
        testing_chat: i64,
        db_controller: &Repo,
    ) -> Result<(Chat, Vehicle), BotDbError> {
        // Añadir a uno vacío
        db_controller
            .create_subscription(testing_plate, testing_chat)
            .await
            .unwrap();

        let chat = db_controller.get_chat(&testing_chat).await.unwrap();

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
            .any(|subscriber| subscriber == testing_chat.to_string()));

        println!("CHAT -> {:?}", chat);
        println!("VEHICLE -> {:?}", vehicle);
        Ok((chat, vehicle))
    }

    #[tokio::test]
    async fn test_end_subscription() {
        // Initialize a test database and repository instance
        let db_controller = Repo::new_for_test("test_end_subscription").await.unwrap();

        let testing_plate = "GHI789";
        let testing_chat = 3;

        // Step 1: Create a subscription for setup
        db_controller
            .create_subscription(testing_plate, testing_chat)
            .await
            .expect("Failed to create initial subscription");

        // Verify that the chat is subscribed to the vehicle
        let chat = db_controller.get_chat(&testing_chat).await.unwrap();
        assert!(chat
            .subscribed_vehicles
            .clone()
            .is_some_and(|t| t.contains(testing_plate)));

        let vehicle = db_controller.get_vehicle(testing_plate).await.unwrap();
        assert!(vehicle
            .subscribers_ids
            .clone()
            .unwrap_or_default()
            .split(',')
            .any(|id| id == testing_chat.to_string()));

        // Step 2: Call `end_subscription` to remove the subscription
        let (n_subscribers, n_subscriptions) = db_controller
            .end_subscription(testing_plate, testing_chat)
            .await
            .expect("Failed to end subscription");

        // Verify the expected number of modifications
        assert_eq!(
            n_subscribers, 1,
            "Expected one subscriber to be removed from vehicle"
        );
        assert_eq!(
            n_subscriptions, 1,
            "Expected one subscription to be removed from chat"
        );

        // Step 3: Verify that the chat is no longer subscribed to the vehicle
        let updated_chat = db_controller.get_chat(&testing_chat).await.unwrap();
        assert!(updated_chat
            .subscribed_vehicles
            .clone()
            .is_some_and(|t| !t.contains(testing_plate)));

        // Verify that the vehicle no longer includes the chat in its subscribers
        let updated_vehicle = db_controller.get_vehicle(testing_plate).await.unwrap();
        assert!(!updated_vehicle
            .subscribers_ids
            .clone()
            .unwrap_or_default()
            .split(',')
            .any(|id| id == testing_chat.to_string()));

        // Clean up the test database
        db_controller.cleanup_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn list_my_vehicles() {
        let db_controller = Repo::new_for_test("list_my_vehicles").await.unwrap();

        let _connection = db_controller.get_connection().get().await.unwrap();

        let expected_subscriptions = vec![
            Vehicle::builder()
                .plate("ABC123".to_string())
                .maybe_subscribers_ids(Some(String::from("1,")))
                .maybe_found_at(None)
                .build(),
            Vehicle::builder()
                .plate("DEF456".to_string())
                .maybe_subscribers_ids(Some(String::from("1,2,")))
                .maybe_found_at(None)
                .build(),
        ];
        let testing_chat = 1;

        let chat = db_controller.get_chat(&testing_chat).await.unwrap();

        let subbed = db_controller
            .get_vehicles_by_chat_id(&chat.id)
            .await
            .unwrap();

        for element in expected_subscriptions {
            assert!(subbed.contains(&element));
        }
        db_controller.cleanup_test_db().await.unwrap();
    }

    /// Test for modifying the active state of a vehicle
    #[tokio::test]
    async fn test_modify_found_at_vehicle() {
        let db_controller = Repo::new_for_test("test_modify_found_at_vehicle")
            .await
            .unwrap();

        let connection = db_controller.get_connection().get().await.unwrap();
        let test_datetime = random_datetime();
        // Modify the active state of a vehicle
        let n = db_controller
            .modify_found_at_vehicle("ABC123", test_datetime)
            .await
            .unwrap();

        // Verify that the vehicle's active state was updated
        let row = connection
            .query_one(
                "SELECT found_at FROM vehicles WHERE plate = $1",
                &[&"ABC123"],
            )
            .await
            .unwrap();

        let active: DateTime<Utc> = row.get(0);
        assert_eq!(active, test_datetime);
        assert_eq!(n, 1);
        db_controller.cleanup_test_db().await.unwrap();
    }

    /// Test for modifying the active state of a chat
    #[tokio::test]
    async fn test_modify_active_chat() {
        let db_controller = Repo::new_for_test("test_modify_active_chat").await.unwrap();

        let connection = db_controller.get_connection().get().await.unwrap();

        // Modify the active state of a chat
        let n = db_controller
            .modify_active_chat(&1_i64, true)
            .await
            .unwrap();

        // Verify that the chat's active state was updated
        let row = connection
            .query_one("SELECT active FROM chats WHERE id = $1", &[&1_i64])
            .await
            .unwrap();

        let active: bool = row.get(0);
        assert!(active);
        assert_eq!(n, 1);

        // Test for setting the chat as inactive
        let n = db_controller
            .modify_active_chat(&1_i64, false)
            .await
            .unwrap();

        // Verify the state after modifying it to inactive
        let row = connection
            .query_one("SELECT active FROM chats WHERE id = $1", &[&1_i64])
            .await
            .unwrap();

        let active: bool = row.get(0);
        assert!(!active);
        assert_eq!(n, 1);
        db_controller.cleanup_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_active_subscriptions_from_vehicle() {
        let db_controller = Repo::new_for_test("test_get_active_subscriptions_from_vehicle")
            .await
            .unwrap();

        let connection = db_controller.get_connection().get().await.unwrap();

        // Setup a test vehicle with specific subscribers, some active and some inactive
        let vehicle_plate = "XYZ987";
        let subscribers_ids = "1,2,3,4,"; // Assuming IDs 1, 2, 3, 4 are in the chats table

        // Insert the vehicle with subscribers
        connection
            .execute(
                "INSERT INTO vehicles (plate, subscribers_ids) VALUES ($1, $2)",
                &[&vehicle_plate, &subscribers_ids],
            )
            .await
            .unwrap();

        // Set chat active states
        db_controller
            .modify_active_chat(&1_i64, true)
            .await
            .unwrap();
        db_controller
            .modify_active_chat(&2_i64, false)
            .await
            .unwrap();
        db_controller
            .modify_active_chat(&3_i64, true)
            .await
            .unwrap();
        db_controller
            .modify_active_chat(&4_i64, false)
            .await
            .unwrap();

        // Call the method to test: it should only return IDs 1 and 3
        let active_chats = db_controller
            .get_active_subscriptions_from_vehicle(vehicle_plate)
            .await
            .unwrap();

        // Convert active_chats to a Vec of IDs for easier comparison
        let active_chat_ids: Vec<i64> = active_chats.into_iter().map(|chat| chat.id).collect();

        // Assert the result matches the expected active chat IDs
        assert_eq!(active_chat_ids, vec![1, 3]);
        db_controller.cleanup_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn test_insert_vehicle() {
        let db_controller = Repo::new_for_test("test_insert_vehicle").await.unwrap();

        let test_vehicle = Vehicle {
            plate: "TEST123".to_string(),
            subscribers_ids: Some("123,".to_string()),
            found_at: None,
        };

        match db_controller.insert_vehicle(test_vehicle.clone()).await {
            Ok(vehicle) => {
                assert_eq!(vehicle.plate, test_vehicle.plate);
                assert_eq!(vehicle.subscribers_ids, test_vehicle.subscribers_ids);
                assert!(vehicle.found_at.is_none());
            }
            Err(e) => panic!("Failed to insert vehicle: {:?}", e),
        };
        db_controller.cleanup_test_db().await.unwrap();
    }

    #[tokio::test]
    async fn test_insert_vehicle_by_plate() {
        let db_controller = Repo::new_for_test("test_insert_vehicle_by_plate")
            .await
            .unwrap();

        let plate = "PLATE123";

        match db_controller.insert_vehicle_plate(plate).await {
            Ok(vehicle) => {
                assert_eq!(vehicle.plate, plate);
            }
            Err(e) => panic!("Failed to insert vehicle by plate: {:?}", e),
        }
        db_controller.cleanup_test_db().await.unwrap();
    }
}
