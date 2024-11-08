use bb8_postgres::tokio_postgres::Row;
use chrono::{DateTime, Utc};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, TypedBuilder)]
pub struct Vehicle {
    pub plate: String,
    pub subscribers_ids: Option<String>,
    //Active == subscribers.is_some_and_not_empty && found_at.is_none
    pub found_at: Option<DateTime<Utc>>,
}

impl From<Row> for Vehicle {
    fn from(row: Row) -> Vehicle {
        Vehicle::builder()
            .plate(row.get("plate"))
            .subscribers_ids(row.try_get("subscribers_ids").ok())
            .found_at(row.try_get("found_at").ok())
            .build()
    }
}

impl PartialEq<Self> for Vehicle {
    fn eq(&self, other: &Self) -> bool {
        self.plate == other.plate
            && self.subscribers_ids == other.subscribers_ids
            && self.found_at == other.found_at
    }
}
