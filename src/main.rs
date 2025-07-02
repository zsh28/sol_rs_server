mod openapi;
mod routes;
mod json_extractor;

use axum::{
    routing::{get, post},
    Router,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use dotenv::dotenv;
use openapi::ApiDoc;
use routes::{get_balance, receive_message, generate_keypair, create_token, mint_token, sign_message, verify_message, send_sol, send_token, Message};
use std::net::SocketAddr;
use tracing_subscriber;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 Server running at http://{}", addr);

    let app = Router::new()
        .route("/submit", post(|req| async {
            match crate::json_extractor::extract_json_with_error_status::<Message>(req).await {
                Ok(Json(payload)) => receive_message(payload).await.into_response(),
                Err(err) => err.0.into_response(),
            }
        }))
        .route("/balance/{address}", get(get_balance))
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(|req| async {
            let result = crate::json_extractor::extract_json_with_error_status(req).await;
            create_token(result).await
        }))
        .route("/token/mint", post(|req| async {
            let result = crate::json_extractor::extract_json_with_error_status(req).await;
            mint_token(result).await
        }))
        .route("/message/sign", post(|req| async {
            let result = crate::json_extractor::extract_json_with_error_status(req).await;
            sign_message(result).await
        }))
        .route("/message/verify", post(|req| async {
            let result = crate::json_extractor::extract_json_with_error_status(req).await;
            verify_message(result).await
        }))
        .route("/send/sol", post(|req| async {
            let result = crate::json_extractor::extract_json_with_error_status(req).await;
            send_sol(result).await
        }))
        .route("/send/token", post(|req| async {
            let result = crate::json_extractor::extract_json_with_error_status(req).await;
            send_token(result).await
        }))
        .merge(SwaggerUi::new("/").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .fallback_service(get(|| async {
            (StatusCode::NOT_FOUND, "Not Found")
        }));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
