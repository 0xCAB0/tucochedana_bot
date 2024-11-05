use bb8_postgres::tokio_postgres::Row;
use chrono::{DateTime, Utc};
use typed_builder::TypedBuilder;

use crate::db::Repo;


#[derive(Debug, Clone, TypedBuilder)]
pub struct Chat {
    pub id: i64,
    pub user_id: u64,
    pub username: String,
    pub offset: Option<i8>,
    pub selected_text: Option<String>,
    pub active: bool,
    pub selected_profiles: Option<String>,
    pub expiration_date: Option<DateTime<Utc>>,
    pub language_code: Option<String>,
    pub sub_type: Option<String>,
}

impl From<Row> for Chat {
    fn from(row: Row) -> Chat {
        let offset_bytes = row.try_get("offset").ok();

        let offset: Option<i8> = match offset_bytes {
            Some(bytes) => {
                let mut arr = [0u8; 1];
                arr.copy_from_slice(bytes);
                Some(i8::from_le_bytes(arr))
            }
            None => None,
        };

        let bytes: &[u8] = row.get("user_id");
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        let user_id: u64 = Repo::as_u64_le(&arr);

        Chat::builder()
            .id(row.get("id"))
            .user_id(user_id)
            .username(row.get("username"))
            .selected_text(row.try_get("selected_text").ok())
            .offset(offset)
            .active(row.get("active"))
            .selected_profiles(row.try_get("selected_profiles").ok())
            .expiration_date(row.try_get("expiration_date").ok())
            .language_code(row.try_get("language_code").ok())
            .sub_type(row.try_get("sub_type").ok())
            .build()
    }
}
