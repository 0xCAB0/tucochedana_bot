// Main reference https://core.telegram.org/bots/webhooks

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::body::{to_bytes, Body};
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
    // Initialize the app
    let app = app();

    // Create a mock IP address and wrap it in ConnectInfo
    let mock_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    // Build the request with the mock IP address in the request extensions
    let mut request = Request::builder()
        .method("GET")
        .uri("/")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(mock_ip));

    // Send the request to the app and await the response
    let response = app.oneshot(request).await.unwrap();

    // Read and convert the response body to a String
    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

    // Assert that the body content is correct
    let expected_body = format!("Hello! Your IP address is: {}", mock_ip.ip());
    assert_eq!(body_string, expected_body);
}
