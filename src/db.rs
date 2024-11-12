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
    #[error("Chat {0} already subscribed to vehicle '{1}'")]
    AlreadySubscribedError(i64, String),
    #[error("Could end subscription from {0} to vehicle '{1}': {2}")]
    CouldNotEndSubscription(i64, String, String),
}

pub use repo::Repo;

pub mod repo;

pub mod model {
    pub mod chat;
    pub mod client_state;
    pub mod vehicle;
}
