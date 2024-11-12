use bb8_postgres::tokio_postgres::NoTls;
use chrono::{DateTime, Duration, Timelike, Utc};
use diesel::{Connection, PgConnection, RunQueryDsl};
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use fang::{AsyncQueue, FangError};
use rand::Rng;

use crate::{db::*, DATABASE_URL};

fn construct_test_db_url(base_url: &str, test_db_name: &str) -> String {
    // Split the URL at the last `/` to get the base connection string without the original database name
    let pos = base_url.rfind('/').expect("Invalid database URL format");

    // Construct the new test database URL by replacing the original database name
    format!("{}{}", &base_url[..pos + 1], test_db_name)
}

impl Repo {
    #[cfg(test)]
    /// Sets up a new, unique test database for each test, applies migrations, and returns a `Repo` instance.
    pub async fn new_for_test() -> Result<Self, BotDbError> {
        // Load environment variables from `.env`
        dotenvy::dotenv().ok();

        // Generate a unique test database name
        let test_db_name = format!("test_db_{}", rand::random::<u32>());

        let mut connection = PgConnection::establish(&DATABASE_URL)
            .unwrap_or_else(|_| panic!("Error connecting to {:?}", *DATABASE_URL));

        let _ =
            diesel::sql_query(format!("CREATE DATABASE {test_db_name}")).execute(&mut connection);

        // Construct the URL for the test database
        let test_db_url = construct_test_db_url(&DATABASE_URL, &test_db_name);

        let mut test_connection = PgConnection::establish(&test_db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {:?}", test_db_url));

        // Run migrations on the test database
        let result = test_connection
            .run_pending_migrations(FileBasedMigrations::find_migrations_directory().unwrap())
            .unwrap();

        println!("Migrations run ..");
        for res in result {
            println!("{:?}", res);
        }

        let pool = Repo::pool(&DATABASE_URL).await?;
        Ok(Repo {
            pool,
            database_name: Some(test_db_name),
        })
    }

    /// Cleans up the test database by dropping it.
    pub async fn cleanup_test_db(&self) -> Result<(), BotDbError> {
        let db_controller = Repo::new_no_tls().await.unwrap();
        let connection = db_controller.get_connection().get().await.unwrap();

        connection
            .execute(
                "DROP DATABASE ($1)",
                &[&self.database_name.clone().unwrap()],
            )
            .await?;
        Ok(())
    }
}

async fn _clear_database() -> Result<(u64, u64), BotDbError> {
    dotenvy::dotenv().ok();
    let db_controller = Repo::new_no_tls().await.unwrap();
    let connection = db_controller.get_connection().get().await?;

    let n1 = &connection
        .execute("TRUNCATE TABLE chats RESTART IDENTITY CASCADE", &[])
        .await?;
    let n2 = &connection
        .execute("TRUNCATE TABLE vehicles RESTART IDENTITY CASCADE", &[])
        .await?;

    log::info!("Cleared {} chats | {} vehicles", n1, n2);

    Ok((*n1, *n2))
}

pub async fn populate_database(repo: &Repo) -> Result<(), BotDbError> {
    let connection = repo.get_connection().get().await?;

    // Insert 3 test users into the `chats` table using connection.execute
    // Insert chats if not exists, conflict on `id`
    connection
        .execute(
            "INSERT INTO chats (id, user_id, username, language_code, subscribed_vehicles)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO NOTHING",
            &[
                &1_i64,
                &123456_u64.to_le_bytes().to_vec(),
                &"user1",
                &Some("en".to_string()),
                &"ABC123,DEF456,",
            ],
        )
        .await?;

    connection
        .execute(
            "INSERT INTO chats (id, user_id, username, language_code, subscribed_vehicles)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (id) DO NOTHING",
            &[
                &2_i64,
                &234567_u64.to_le_bytes().to_vec(),
                &"user2",
                &Some("fr".to_string()),
                &"DEF456,",
            ],
        )
        .await?;

    connection
        .execute(
            "INSERT INTO chats (id, user_id, username, language_code)
VALUES ($1, $2, $3, $4)
ON CONFLICT (id) DO NOTHING",
            &[
                &3_i64,
                &345678_u64.to_le_bytes().to_vec(),
                &"user3",
                &Some("es".to_string()),
            ],
        )
        .await?;

    // Insert vehicles if not exists, conflict on `plate`
    connection
        .execute(
            "INSERT INTO vehicles (plate, subscribers_ids)
VALUES ($1, $2)
ON CONFLICT (plate) DO NOTHING",
            &[&"ABC123", &"1,"],
        )
        .await?;

    connection
        .execute(
            "INSERT INTO vehicles (plate, subscribers_ids)
VALUES ($1, $2)
ON CONFLICT (plate) DO NOTHING",
            &[&"DEF456", &"1,2,"],
        )
        .await?;

    connection
        .execute(
            "INSERT INTO vehicles (plate)
VALUES ($1)
ON CONFLICT (plate) DO NOTHING",
            &[&"GHI789"],
        )
        .await?;

    Ok(())
}

pub fn random_datetime() -> DateTime<Utc> {
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

pub async fn create_mock_queue() -> Result<AsyncQueue<NoTls>, FangError> {
    let mut queue: AsyncQueue<NoTls> = AsyncQueue::builder()
        .uri(DATABASE_URL.to_string())
        .max_pool_size(5_u32)
        .build();

    queue.connect(NoTls).await.unwrap();
    Ok(queue)
}
