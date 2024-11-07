use db::BotDbError;
use fang::{AsyncQueueError, FangError, ToFangError};
use frankenstein::reqwest::StatusCode;
use lazy_static::lazy_static;
use std::fmt::{self, Debug};
use telegram::client::ApiError;
use thiserror::Error;

// Environment variables
lazy_static! {
    pub static ref TELEGRAM_BOT_TOKEN: String =
        std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    pub static ref DATABASE_URL: String =
        std::env::var("DATABASE_URL").expect("DATABASE URL not set");
    pub static ref API_URL: String = std::env::var("API_URL").expect("API_URL not set");
    pub static ref WEBHOOK_URL: String = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL not set");
    pub static ref WEBHOOK_CERT: Option<String> = std::env::var("WEBHOOK_CERT").ok();
    pub static ref WEBHOOK_PORT: u32 = std::env::var("WEBHOOK_PORT")
        .expect("WEBHOOK_PORT not set")
        .parse()
        .expect("WEBHOOK_PORT should be an u32 number");
    pub static ref SERVER_PORT: u32 = std::env::var("SERVER_PORT")
        .expect("SERVER_PORT not set")
        .parse()
        .expect("SERVER_PORT should be an u32 number");
    pub static ref BOT_NAME: String = std::env::var("BOT_NAME").expect("BOT_NAME not set");
    pub static ref FETCH_IN_MINUTES: u8 = std::env::var("FETCH_IN_MINUTES")
        .unwrap_or(String::from("5"))
        .parse()
        .expect("The number of minutes should be 0<=N<=255");
    pub static ref MAX_RETRIES: i32 = std::env::var("MAX_RETRIES")
        .unwrap_or(String::from("1"))
        .parse()
        .expect("The number of retries should be 0<=N<=255");
}

const TASK_NAME: &str = "scheduled_fetch";

/// HTTP Server module
pub mod server;

/// Fang task
pub mod workers;

/// API Module
pub mod tucochedana {
    pub mod client;
}

/// Database module
pub mod db;

pub mod telegram {
    pub mod client;
}

pub mod update_handler {
    pub mod command;
    pub mod process_update;
}

pub mod tasks {
    pub mod fetch;
}

#[derive(Debug, Error, ToFangError)]
pub enum BotError {
    #[error(transparent)]
    MessageError(#[from] std::fmt::Error),
    #[error("Update can not be processed {}", self)]
    UpdateNotMessage(String),
    #[error(transparent)]
    TelegramError(#[from] ApiError),
    #[error(transparent)]
    DbError(#[from] BotDbError),
    #[error(transparent)]
    ReqwestError(#[from] frankenstein::reqwest::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] SerdeJSONError),
    #[error("Api returned code {0}: {1}")]
    TuCocheDanaError(StatusCode, String),
    #[error(transparent)]
    HttpError(#[from] std::io::Error),
    #[error(transparent)]
    AsyncQueueError(#[from] AsyncQueueError),
}

#[derive(Debug, Error)]
pub struct SerdeJSONError {
    raw_json: String,
    serde_error: serde_json::Error,
}

impl SerdeJSONError {
    fn _new(raw_json: String, serde_error: serde_json::Error) -> Self {
        SerdeJSONError {
            raw_json,
            serde_error,
        }
    }
}

impl fmt::Display for SerdeJSONError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JSON: {}, SerdeError: {}",
            self.raw_json, self.serde_error
        )
    }
}
