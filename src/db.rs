use bb8_postgres::bb8::RunError;
use fang::FangError;
use fang::ToFangError;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error, ToFangError)]
pub enum BotDbError {
    #[error(transparent)]
    PoolError(#[from] RunError<bb8_postgres::tokio_postgres::Error>),
    #[error(transparent)]
    PgError(#[from] bb8_postgres::tokio_postgres::Error),
    #[error(transparent)]
    CronError(#[from] cron::error::Error),
    #[error("City not found")]
    CityNotFoundError,
    #[error("No timestamps that match with this cron expression")]
    NoTimestampsError,
}


pub use repo::Repo;

pub mod repo;

pub mod model{
    pub mod vehicle;
    pub mod chat;
}