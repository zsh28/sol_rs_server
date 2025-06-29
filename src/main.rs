use axum::{
    extract::{Path, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use std::str::FromStr;


#[derive(Debug, Deserialize, Serialize)]
struct Message {
    name: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct Response {
    status: String,
    echoed: Message,
}

#[derive(Debug, Serialize)]
struct BalanceResponse {
    address: String,
    lamports: u64,
    sol: f64,
}

async fn hello() -> &'static str {
    "Hello from Rust!"
}

async fn receive_message(Json(payload): Json<Message>) -> Json<Response> {
    let response = Response {
        status: "Received".to_string(),
        echoed: payload,
    };

    Json(response)
}

async fn get_balance(Path(address): Path<String>) -> Json<BalanceResponse> {
    // Use Solana devnet
    let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    let pubkey = Pubkey::from_str(&address).unwrap_or(Pubkey::default());

    let lamports = client.get_balance(&pubkey).unwrap_or(0);
    let sol = lamports as f64 / 1_000_000_000.0;

    Json(BalanceResponse {
        address,
        lamports,
        sol,
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello))
        .route("/submit", post(receive_message))
        .route("/balance/{address}", get(get_balance));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
