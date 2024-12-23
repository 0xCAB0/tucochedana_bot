use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use fang::NoTls;

use fang::AsyncQueue;
use frankenstein::Update;
use tokio::sync::Mutex;

use crate::{update_handler::process_update::UpdateProcessor, BotError};

pub fn app(queue: AsyncQueue<NoTls>) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello!" }))
        .route("/webhook", post(parse_update))
        .with_state(Arc::new(Mutex::new(queue)))
}
async fn parse_update(
    State(state): State<Arc<Mutex<AsyncQueue<NoTls>>>>,
    Json(update): Json<Update>,
) -> axum::response::Result<()> {
    UpdateProcessor::run(&update, state).await.unwrap();
    Ok(())
}

impl IntoResponse for BotError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, format!("{self}")).into_response()
    }
}

// Main reference https://core.telegram.org/bots/webhooks
#[cfg(test)]
mod server_tests {

    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use frankenstein::Chat;
    use frankenstein::Message;
    use frankenstein::Update;
    use frankenstein::UpdateContent;
    use frankenstein::User;
    use tower::ServiceExt;

    use super::*;
    use crate::db::Repo;

    /// Basic example https://core.telegram.org/bots/webhooks#testing-your-bot-with-updates
    #[ignore = "Unestable"]
    #[tokio::test]
    async fn test_webhook_dispatch() {
        dotenvy::dotenv().ok();

        let repo = Repo::repo().await.unwrap();

        let queue = Repo::create_testing_queue(repo, true).await.unwrap();
        // Initialize the app
        let app = app(queue);

        let db_chat = repo.get_testing_chat().await.unwrap();

        let chat = Chat::builder()
            .id(db_chat.user_id as i64)
            .type_field(frankenstein::ChatType::Private)
            .username("Test".to_string())
            .first_name("Test".to_string())
            .last_name("Test Lastname".to_string())
            .build();

        let from = User::builder()
            .id(db_chat.user_id)
            .is_bot(false)
            .username(chat.username.clone().unwrap())
            .first_name(chat.first_name.clone().unwrap())
            .last_name(chat.last_name.clone().unwrap())
            .build();

        let message: Message = Message::builder()
            .date(1441645532)
            .chat(chat)
            .message_id(382)
            .from(from)
            .text("/start")
            .build();

        let content: UpdateContent = UpdateContent::Message(message);
        let update = Update::builder().update_id(10000).content(content).build();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhook")
                    .header("Content-Type", "application/json")
                    .header("Cache-Control", "no-cache")
                    .body(serde_json::to_string(&update).unwrap())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Response: {:#?}",
            response.body()
        );
    }

    #[tokio::test]
    async fn test_root_handler() {
        dotenvy::dotenv().ok();

        let queue = Repo::create_testing_queue(Repo::repo().await.unwrap(), true)
            .await
            .unwrap();
        // Initialize the app
        let app = app(queue);

        // Build the request with the mock IP address in the request extensions
        let request = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        // Send the request to the app and await the response
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Response: {:?}",
            response.body()
        );
    }
}
