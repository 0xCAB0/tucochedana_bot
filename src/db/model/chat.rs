use bb8_postgres::tokio_postgres::Row;
use bon::Builder;

use crate::db::Repo;

use super::client_state::ClientState;

#[derive(Debug, Clone, Builder)]
pub struct Chat {
    pub id: i64,
    pub user_id: u64,
    pub username: String,
    pub state: ClientState,
    pub selected_text: Option<String>,
    pub subscribed_vehicles: Option<String>,
    pub active: bool,
    pub language_code: Option<String>,
}

impl From<Row> for Chat {
    fn from(row: Row) -> Chat {
        let bytes: &[u8] = row.get("user_id");
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        let user_id: u64 = Repo::as_u64_le(&arr);

        Chat::builder()
            .id(row.get("id"))
            .user_id(user_id)
            .username(row.get("username"))
            .state(row.get("state"))
            .maybe_selected_text(row.try_get("selected_text").ok())
            .maybe_subscribed_vehicles(row.try_get("subscribed_vehicles").ok())
            .active(row.get("active"))
            .maybe_language_code(row.try_get("language_code").ok())
            .build()
    }
}
