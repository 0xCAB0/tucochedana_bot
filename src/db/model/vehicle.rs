use bb8_postgres::tokio_postgres::Row;
use chrono::{DateTime, Utc};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder, PartialEq)]
pub struct Vehicle {
    pub plate: String,
    pub subscribers_ids: Option<String>,
    pub active: bool,
    pub found_at: Option<DateTime<Utc>>,
}

impl From<Row> for Vehicle {
    fn from(row: Row) -> Vehicle {
        Vehicle::builder()
            .plate(row.get("plate"))
            .subscribers_ids(row.try_get("subscribers_ids").ok())
            .active(row.get("active"))
            .found_at(row.try_get("found_at").ok())
            .build()
    }
}
