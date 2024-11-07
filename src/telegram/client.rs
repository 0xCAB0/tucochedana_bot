use crate::TELEGRAM_BOT_TOKEN;
use fang::FangError;
use fang::ToFangError;
use frankenstein::AllowedUpdate;
use frankenstein::AnswerPreCheckoutQueryParams;
use frankenstein::AsyncApi;
use frankenstein::AsyncTelegramApi;
use frankenstein::ChatAction;
use frankenstein::DeleteWebhookParams;
use frankenstein::EditMessageReplyMarkupParams;
use frankenstein::EditMessageTextParams;
use frankenstein::FileUpload;
use frankenstein::GetStickerSetParams;
use frankenstein::GetUpdatesParams;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::InputFile;
use frankenstein::Message;
use frankenstein::MessageOrBool;
use frankenstein::MethodResponse;
use frankenstein::ParseMode;
use frankenstein::ReplyMarkup;
use frankenstein::ReplyParameters;
use frankenstein::SendChatActionParams;
use frankenstein::SendMessageParams;
use frankenstein::SendStickerParams;
use frankenstein::SendVideoParams;
use frankenstein::SetWebhookParams;
use frankenstein::StickerSet;
use frankenstein::Update;
use frankenstein::WebhookInfo;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use tokio::sync::OnceCell;

use std::fmt::Debug;

static API_CLIENT: OnceCell<ApiClient> = OnceCell::const_new();

#[derive(Debug, Error, ToFangError)]
pub enum ApiError {
    #[error(transparent)]
    FrankensteinError(#[from] frankenstein::Error),
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    telegram_client: AsyncApi,
    update_params: GetUpdatesParams,
    buffer: VecDeque<Update>,
}

#[derive(Debug, Clone)]
pub enum Buttons {
    PassToButtons(Vec<Vec<InlineKeyboardButton>>),
    Buttons(InlineKeyboardMarkup),
}

impl From<Vec<Vec<InlineKeyboardButton>>> for Buttons {
    fn from(value: Vec<Vec<InlineKeyboardButton>>) -> Self {
        Buttons::PassToButtons(value)
    }
}

impl From<InlineKeyboardMarkup> for Buttons {
    fn from(value: InlineKeyboardMarkup) -> Self {
        Buttons::Buttons(value)
    }
}

impl ApiClient {
    pub async fn api_client() -> &'static Self {
        API_CLIENT.get_or_init(ApiClient::new).await
    }

    pub async fn new() -> Self {
        let telegram_client = AsyncApi::new(&TELEGRAM_BOT_TOKEN);

        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![
                AllowedUpdate::Message,
                //AllowedUpdate::ChannelPost,
                AllowedUpdate::CallbackQuery,
            ])
            .build();

        let buffer = VecDeque::new();

        Self {
            telegram_client,
            update_params,
            buffer,
        }
    }

    pub async fn set_webhook(
        &self,
        url: &String,
        ip_address: Option<String>,
        certificate_path: Option<String>,
    ) -> Result<MethodResponse<bool>, ApiError> {
        let file: Option<InputFile> = certificate_path.map(|path| {
            InputFile::builder()
                .path(PathBuf::from_str(path.as_str()).unwrap())
                .build()
        });

        let params: SetWebhookParams = match ip_address {
            Some(ip) => SetWebhookParams::builder()
                .url(url)
                .ip_address(ip)
                .maybe_certificate(file)
                .allowed_updates(self.update_params.allowed_updates.clone().unwrap())
                .build(),
            None => SetWebhookParams::builder()
                .url(url)
                .allowed_updates(self.update_params.allowed_updates.clone().unwrap())
                .build(),
        };

        Ok(self.telegram_client.set_webhook(&params).await?)
    }

    pub async fn remove_webhook(&self) -> Result<MethodResponse<bool>, ApiError> {
        let params: DeleteWebhookParams = DeleteWebhookParams::builder()
            .drop_pending_updates(false)
            .build();

        Ok(self.telegram_client.delete_webhook(&params).await?)
    }

    pub async fn get_webhook_info(&self) -> Result<MethodResponse<WebhookInfo>, ApiError> {
        Ok(self.telegram_client.get_webhook_info().await?)
    }

    pub async fn next_update(&mut self) -> Option<Update> {
        if let Some(update) = self.buffer.pop_front() {
            return Some(update);
        }

        match self.telegram_client.get_updates(&self.update_params).await {
            Ok(updates) => {
                for update in updates.result {
                    self.buffer.push_back(update);
                }

                if let Some(last_update) = self.buffer.back() {
                    self.update_params.offset = Some((last_update.update_id + 1).into());
                }

                self.buffer.pop_front()
            }

            Err(err) => {
                log::error!("Failed to fetch updates {:?}", err);
                None
            }
        }
    }

    pub async fn send_typing(&self, chat_id: i64) -> Result<MethodResponse<bool>, ApiError> {
        let send_chat_action_params = SendChatActionParams::builder()
            .chat_id(chat_id)
            .action(ChatAction::Typing)
            .build();

        Ok(self
            .telegram_client
            .send_chat_action(&send_chat_action_params)
            .await?)
    }

    pub async fn edit_or_send_message(
        &self,
        chat_id: i64,
        message_id: i32,
        text: &str,
        inline_keyboard: InlineKeyboardMarkup,
    ) -> Result<(), ApiError> {
        //1. try editing
        if self
            .edit_message(chat_id, message_id, text, inline_keyboard.clone())
            .await
            .is_err()
        {
            // on failure -> Send message
            self.send_message_with_buttons(chat_id, text, inline_keyboard)
                .await?;
        }

        Ok(())
    }

    pub async fn send_message_with_buttons(
        &self,
        chat_id: i64,
        text: &str,
        inline_keyboard: InlineKeyboardMarkup,
    ) -> Result<MethodResponse<Message>, ApiError> {
        let message_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(inline_keyboard))
            .parse_mode(ParseMode::Html)
            .build();

        Ok(self.telegram_client.send_message(&message_params).await?)
    }

    pub async fn send_message(
        &self,
        chat_id: i64,
        message_id: i32,
        text: String,
    ) -> Result<MethodResponse<Message>, ApiError> {
        let send_message_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .reply_parameters(ReplyParameters::builder().message_id(message_id).build())
            .parse_mode(ParseMode::Html)
            .build();

        Ok(self
            .telegram_client
            .send_message(&send_message_params)
            .await?)
    }

    pub async fn send_message_without_reply(
        &self,
        chat_id: i64,
        text: String,
    ) -> Result<MethodResponse<Message>, ApiError> {
        let send_message_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(text)
            .parse_mode(ParseMode::Html)
            .build();

        Ok(self
            .telegram_client
            .send_message(&send_message_params)
            .await?)
    }

    pub async fn approve_payment(
        &self,
        checkout_id: &String,
    ) -> Result<MethodResponse<bool>, ApiError> {
        let params = AnswerPreCheckoutQueryParams::builder()
            .ok(true)
            .pre_checkout_query_id(checkout_id)
            .build();

        Ok(self
            .telegram_client
            .answer_pre_checkout_query(&params)
            .await?)
    }

    pub async fn get_sticker_set(&self, name: &str) -> Result<StickerSet, ApiError> {
        let params = GetStickerSetParams::builder().name(name).build();
        Ok(self.telegram_client.get_sticker_set(&params).await?.result)
    }

    pub async fn send_sticker_message(
        &self,
        chat_id: i64,
        file_id: &str,
    ) -> Result<Message, ApiError> {
        let params = SendStickerParams::builder()
            .sticker(file_id.to_string())
            .chat_id(chat_id)
            .build();

        Ok(self.telegram_client.send_sticker(&params).await?.result)
    }

    pub async fn send_video_with_text(
        &self,
        chat_id: i64,
        video: &str,
        message: &String,
    ) -> Result<Message, ApiError> {
        let params = SendVideoParams::builder()
            .video(FileUpload::from(video.to_string()))
            .caption(message)
            .chat_id(chat_id)
            .build();

        Ok(self.telegram_client.send_video(&params).await?.result)
    }

    async fn edit_message(
        &self,
        chat_id: i64,
        message_id: i32,
        text: &str,
        inline_keyboard: InlineKeyboardMarkup,
    ) -> Result<MethodResponse<MessageOrBool>, ApiError> {
        let edit_text = EditMessageTextParams::builder()
            .chat_id(chat_id)
            .message_id(message_id)
            .text(text)
            .parse_mode(ParseMode::Html)
            .build();

        let edit_keyboard = EditMessageReplyMarkupParams::builder()
            .chat_id(chat_id)
            .message_id(message_id)
            .reply_markup(inline_keyboard)
            .build();

        self.telegram_client.edit_message_text(&edit_text).await?;

        Ok(self
            .telegram_client
            .edit_message_reply_markup(&edit_keyboard)
            .await?)
    }
}
