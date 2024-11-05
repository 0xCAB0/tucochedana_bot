use bb8_postgres::tokio_postgres::Row;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Vehicle {
    pub plate: String,
    pub chat_id: i64,
}

impl From<Row> for Vehicle {
    fn from(row: Row) -> Vehicle {
        Vehicle::builder()
            .plate(row.get("plate"))
            .chat_id(row.get("chat_id"))
            .build()
    }
}
