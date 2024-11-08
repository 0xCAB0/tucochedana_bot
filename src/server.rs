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
    UpdateProcessor::run(&update, state).await?;
    Ok(())
}

impl IntoResponse for BotError {
    fn into_response(self) -> axum::response::Response {
        // Customize error response here
        (StatusCode::BAD_REQUEST, format!("{self}")).into_response()
    }
}

// Main reference https://core.telegram.org/bots/webhooks
#[cfg(test)]
mod server_tests {

    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::StatusCode;
    use fang::AsyncQueue;
    use fang::NoTls;
    use frankenstein::Chat;
    use frankenstein::Message;
    use frankenstein::Update;
    use frankenstein::UpdateContent;
    use frankenstein::User;
    use lambda_http::tower::ServiceExt;

    use super::*;
    use crate::DATABASE_URL;

    async fn init_testing_queue() -> AsyncQueue<NoTls> {
        let mut queue: AsyncQueue<NoTls> = AsyncQueue::builder()
            .uri(DATABASE_URL.clone())
            .max_pool_size(1_u32)
            .build();

        queue.connect(NoTls).await.unwrap();
        queue
    }

    /// Basic example https://core.telegram.org/bots/webhooks#testing-your-bot-with-updates
    #[tokio::test]
    async fn test_webhook_dispatch() {
        dotenvy::dotenv().ok();

        let queue = init_testing_queue().await;
        let app = app(queue);

        let chat = Chat::builder()
            .id(1111111)
            .type_field(frankenstein::ChatType::Private)
            .username("Test".to_string())
            .first_name("Test".to_string())
            .last_name("Test Lastname".to_string())
            .build();

        let from = User::builder()
            .id(1111111)
            .is_bot(false)
            .username("Test".to_string())
            .first_name("Test".to_string())
            .last_name("Test Lastname".to_string())
            .build();

        let message: Message = Message::builder()
            .date(1441645532)
            .chat(chat)
            .message_id(1365)
            .from(from)
            .text("/start")
            .build();

        let content: UpdateContent = UpdateContent::Message(message);
        let update = Update::builder().update_id(10000).content(content).build();

        let result = app
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

        assert_eq!(result.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_root_handler() {
        dotenvy::dotenv().ok();

        let queue = init_testing_queue().await;
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
        assert_eq!(response.status(), StatusCode::OK);
    }
}
