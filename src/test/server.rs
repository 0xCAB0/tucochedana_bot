// Main reference https://core.telegram.org/bots/webhooks

use axum::http::{Request, StatusCode};
use frankenstein::{Chat, Message, Update, UpdateContent, User};
use lambda_http::tower::ServiceExt;

use crate::server::app;

/// Basic example https://core.telegram.org/bots/webhooks#testing-your-bot-with-updates
#[tokio::test]
async fn test_webhook_dispatch() {
    dotenvy::dotenv().ok();

    let app = app();

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
                .uri("/")
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-cache")
                .body(serde_json::to_string(&update).unwrap())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(result.status(), StatusCode::OK);
}
