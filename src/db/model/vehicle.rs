use bb8_postgres::tokio_postgres::Row;
use bon::Builder;
use bytes::{BufMut, BytesMut};
use chrono::{DateTime, Datelike, Timelike, Utc};
use postgres_types::{IsNull, ToSql, Type};
use std::{error::Error, fmt::Debug};

#[derive(Debug, Clone, Builder)]
pub struct Vehicle {
    pub plate: String,
    pub subscribers_ids: Option<String>,
    //Active == subscribers.is_some_and_not_empty && found_at.is_none
    pub found_at: Option<DateTime<Utc>>,
}

impl ToSql for Vehicle {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        // Serialize each field as PostgreSQL expects
        // Convert `plate` to SQL
        out.put_slice(self.plate.as_bytes());

        // Serialize `subscribers_ids` as a nullable value
        match &self.subscribers_ids {
            Some(subscribers) => {
                out.put_slice(subscribers.as_bytes());
            }
            None => return Ok(IsNull::Yes), // Mark as null if empty
        }

        // Serialize `found_at` as a nullable timestamp if provided
        if let Some(found) = &self.found_at {
            let timestamp = found.timestamp().to_string();
            out.put_slice(timestamp.as_bytes());
        } else {
            return Ok(IsNull::Yes);
        }

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized,
    {
        matches!(ty.name(), "text" | "varchar" | "timestamp")
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        // Perform additional checks or validations before serializing
        self.to_sql(ty, out)
    }
}

impl From<Row> for Vehicle {
    fn from(row: Row) -> Vehicle {
        Vehicle::builder()
            .plate(row.get("plate"))
            .maybe_subscribers_ids(row.try_get("subscribers_ids").ok())
            .maybe_found_at(row.try_get("found_at").ok())
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

impl Vehicle {
    pub fn found_at_to_text(&self) -> String {
        // Spanish names for days of the week
        let days = [
            "domingo",
            "lunes",
            "martes",
            "miércoles",
            "jueves",
            "viernes",
            "sábado",
        ];
        // Spanish names for months
        let months = [
            "enero",
            "febrero",
            "marzo",
            "abril",
            "mayo",
            "junio",
            "julio",
            "agosto",
            "septiembre",
            "octubre",
            "noviembre",
            "diciembre",
        ];

        let Some(time) = &self.found_at else {
            return format!("El vehículo {} no ha sido encontrado todavía", self.plate);
        };

        // Get day of the week, day of the month, month, and year
        let weekday = days[time.weekday().num_days_from_sunday() as usize];
        let day = time.day();
        let month = months[(time.month() - 1) as usize];
        let year = time.year();
        let hour = time.hour();
        let minute = time.minute();

        // Format the date as a Spanish-readable string
        format!(
            "El vehículo {} fue encontrado el {}, {} de {} de {}, {:02}:{:02} 🙌🏼",
            self.plate, weekday, day, month, year, hour, minute
        )
    }
}
