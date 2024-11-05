use std::fmt::Debug;
use db::BotDbError;
use lazy_static::lazy_static;
use telegram::client::ApiError;
use thiserror::Error;
use fang::{ToFangError, FangError};
lazy_static!{
    pub static ref TELEGRAM_BOT_TOKEN: String =
        std::env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    pub static ref DATABASE_URL: String =
        std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    pub static ref API_URL: String = 
        std::env::var("API_URL").expect("API_URL not set");
    pub static ref BOT_NAME: String = 
        std::env::var("BOT_NAME").expect("BOT_NAME not set");
}

pub mod tucochedana{
    pub mod client;
}

pub mod db;

pub mod telegram {
    pub mod client;
    pub mod handler;
}

pub mod update_handler {
    pub mod command;
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
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}