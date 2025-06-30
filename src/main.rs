mod openapi;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use openapi::ApiDoc;
use routes::{get_balance, hello, receive_message};
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
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
