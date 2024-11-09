use chrono::{DateTime, Duration, Timelike, Utc};
use rand::Rng;

use crate::db::*;

pub async fn setup() -> Result<(), BotDbError> {
    clear_database().await?;
    populate_database().await?;
    Ok(())
}

async fn clear_database() -> Result<(u64, u64), BotDbError> {
    dotenvy::dotenv().ok();
    let db_controller = Repo::new_no_tls().await.unwrap();
    let connection = db_controller.get_connection().get().await?;

    let n1 = &connection.execute("DELETE FROM chats", &[]).await?;
    let n2 = &connection.execute("DELETE FROM vehicles", &[]).await?;

    log::info!("Cleared {} chats | {} vehicles", n1, n2);

    Ok((*n1, *n2))
}

async fn populate_database() -> Result<(), BotDbError> {
    dotenvy::dotenv().ok();
    let db_controller = Repo::new_no_tls().await.unwrap();
    let connection = db_controller.get_connection().get().await?;

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
