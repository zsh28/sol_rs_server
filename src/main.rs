mod openapi;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use openapi::ApiDoc;
use routes::{get_balance, hello, receive_message, generate_keypair, create_token, mint_token, sign_message, verify_message, send_sol, send_token};
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
        .route("/", get(hello))
        .route("/submit", post(receive_message))
        .route("/balance/{address}", get(get_balance))
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/sign", post(sign_message))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token))
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
