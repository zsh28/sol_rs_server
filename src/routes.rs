use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct Message {
    name: String,
    message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Response {
    status: String,
    echoed: Message,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BalanceResponse {
    address: String,
    lamports: u64,
    sol: f64,
}

/// GET /
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Health check")
    )
)]
pub async fn hello() -> &'static str {
    "Hello from Rust!"
}

/// POST /submit
#[utoipa::path(
    post,
    path = "/submit",
    request_body = Message,
    responses(
        (status = 200, description = "Echoed message", body = Response)
    )
)]
pub async fn receive_message(Json(payload): Json<Message>) -> Json<Response> {
    let response = Response {
        status: "Received".to_string(),
        echoed: payload,
    };

    Json(response)
}

/// GET /balance/:address
#[utoipa::path(
    get,
    path = "/balance/{address}",
    params(
        ("address" = String, Path, description = "Solana wallet address")
    ),
    responses(
        (status = 200, description = "Account balance", body = BalanceResponse),
        (status = 400, description = "Invalid address format"),
        (status = 500, description = "Failed to fetch balance")
    )
)]
pub async fn get_balance(Path(address): Path<String>) -> impl IntoResponse {
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let client = RpcClient::new(rpc_url);

    let pubkey = match Pubkey::from_str(&address) {
        Ok(pk) => pk,
        Err(_) => {
            tracing::warn!("Invalid pubkey: {}", address);
            return (StatusCode::BAD_REQUEST, "Invalid address format").into_response();
        }
    };

    match client.get_balance(&pubkey) {
        Ok(lamports) => {
            let sol = lamports as f64 / 1_000_000_000.0;
            Json(BalanceResponse {
                address,
                lamports,
                sol,
            })
            .into_response()
        }
        Err(e) => {
            tracing::error!("RPC error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch balance").into_response()
        }
    }
}
