use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine as _};
use bs58;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
};
use spl_token::instruction::{initialize_mint, mint_to, transfer as token_transfer};
use std::str::FromStr;
use utoipa::ToSchema;
use solana_sdk::system_instruction;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success { success: bool, data: T },
    Error { success: bool, error: String },
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(self);
        (StatusCode::OK, body).into_response()
    }
}

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

#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenCreateRequest {
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TokenMintRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MessageSignRequest {
    message: String,
    secret: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MessageVerifyRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyMessageResponse {
    valid: bool,
    message: String,
    pubkey: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

#[utoipa::path(get, path = "/")]
pub async fn hello() -> &'static str {
    "Hello from Rust!"
}

#[utoipa::path(post, path = "/submit")]
pub async fn receive_message(Json(payload): Json<Message>) -> Json<Response> {
    Json(Response {
        status: "Received".to_string(),
        echoed: payload,
    })
}

#[utoipa::path(get, path = "/balance/{address}")]
pub async fn get_balance(Path(address): Path<String>) -> impl IntoResponse {
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let client = RpcClient::new(rpc_url);

    let pubkey = match Pubkey::from_str(&address) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid address format".to_string(),
            }
            .into_response()
        }
    };

    match client.get_balance(&pubkey) {
        Ok(lamports) => ApiResponse::Success {
            success: true,
            data: BalanceResponse {
                address,
                lamports,
                sol: lamports as f64 / 1_000_000_000.0,
            },
        }
        .into_response(),
        Err(_) => ApiResponse::<()>::Error {
            success: false,
            error: "Failed to fetch balance".to_string(),
        }
        .into_response(),
    }
}

#[utoipa::path(post, path = "/keypair")]
pub async fn generate_keypair() -> axum::response::Response {
    let keypair = Keypair::new();
    ApiResponse::Success {
        success: true,
        data: serde_json::json!({
            "pubkey": keypair.pubkey().to_string(),
            "secret": bs58::encode(keypair.to_bytes()).into_string(),
        }),
    }
    .into_response()
}

#[utoipa::path(post, path = "/token/create")]
pub async fn create_token(Json(req): Json<TokenCreateRequest>) -> axum::response::Response {
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(p) => p,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid mint address".to_string(),
        }.into_response(),
    };

    let authority = match Pubkey::from_str(&req.mint_authority) {
        Ok(p) => p,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid mint authority address".to_string(),
        }.into_response(),
    };

    let ix = match initialize_mint(&spl_token::id(), &mint, &authority, None, req.decimals) {
        Ok(instr) => instr,
        Err(e) => return ApiResponse::<()>::Error {
            success: false,
            error: format!("Failed to create instruction: {}", e),
        }.into_response(),
    };

    ApiResponse::Success {
        success: true,
        data: serde_json::json!({
            "program_id": ix.program_id.to_string(),
            "accounts": ix.accounts.iter().map(|a| serde_json::json!({
                "pubkey": a.pubkey.to_string(),
                "is_signer": a.is_signer,
                "is_writable": a.is_writable,
            })).collect::<Vec<_>>(),
            "instruction_data": general_purpose::STANDARD.encode(ix.data),
        }),
    }.into_response()
}

#[utoipa::path(post, path = "/token/mint")]
pub async fn mint_token(Json(req): Json<TokenMintRequest>) -> axum::response::Response {
    let mint = Pubkey::from_str(&req.mint).unwrap();
    let destination = Pubkey::from_str(&req.destination).unwrap();
    let authority = Pubkey::from_str(&req.authority).unwrap();

    let ix = mint_to(&spl_token::id(), &mint, &destination, &authority, &[], req.amount).unwrap();

    ApiResponse::Success {
        success: true,
        data: serde_json::json!({
            "program_id": ix.program_id.to_string(),
            "accounts": ix.accounts.iter().map(|a| serde_json::json!({
                "pubkey": a.pubkey.to_string(),
                "is_signer": a.is_signer,
                "is_writable": a.is_writable,
            })).collect::<Vec<_>>(),
            "instruction_data": general_purpose::STANDARD.encode(ix.data),
        }),
    }.into_response()
}

#[utoipa::path(post, path = "/message/sign")]
pub async fn sign_message(Json(req): Json<MessageSignRequest>) -> axum::response::Response {
    let secret_bytes = bs58::decode(&req.secret).into_vec().unwrap();
    let keypair = Keypair::from_bytes(&secret_bytes).unwrap();
    let signature = keypair.sign_message(req.message.as_bytes());

    ApiResponse::Success {
        success: true,
        data: serde_json::json!({
            "signature": general_purpose::STANDARD.encode(signature.as_ref()),
            "public_key": keypair.pubkey().to_string(),
            "message": req.message,
        }),
    }.into_response()
}

#[utoipa::path(post, path = "/message/verify")]
pub async fn verify_message(Json(req): Json<MessageVerifyRequest>) -> axum::response::Response {
    let pubkey = match Pubkey::from_str(&req.pubkey) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid pubkey format".to_string(),
        }.into_response(),
    };

    let signature_bytes = match general_purpose::STANDARD.decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid signature encoding".to_string(),
        }.into_response(),
    };

    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Failed to parse signature".to_string(),
        }.into_response(),
    };

    let valid = signature.verify(pubkey.as_ref(), req.message.as_bytes());

    ApiResponse::Success {
        success: true,
        data: VerifyMessageResponse {
            valid,
            message: req.message,
            pubkey: req.pubkey,
        },
    }
    .into_response()
}

#[utoipa::path(post, path = "/send/sol")]
pub async fn send_sol(Json(req): Json<SendSolRequest>) -> axum::response::Response {
    let from = match Pubkey::from_str(&req.from) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid sender address".to_string(),
        }.into_response(),
    };
    let to = match Pubkey::from_str(&req.to) {
        Ok(pk) => pk,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid recipient address".to_string(),
        }.into_response(),
    };

    let ix = system_instruction::transfer(&from, &to, req.lamports);

    ApiResponse::Success {
        success: true,
        data: serde_json::json!({
            "program_id": ix.program_id.to_string(),
            "accounts": ix.accounts.iter().map(|a| a.pubkey.to_string()).collect::<Vec<_>>(),
            "instruction_data": general_purpose::STANDARD.encode(ix.data),
        }),
    }.into_response()
}

#[utoipa::path(post, path = "/send/token")]
pub async fn send_token(Json(req): Json<SendTokenRequest>) -> axum::response::Response {
    let destination = match Pubkey::from_str(&req.destination) {
        Ok(p) => p,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid destination address".to_string(),
        }.into_response(),
    };

    let _mint = match Pubkey::from_str(&req.mint) {
        Ok(p) => p,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid mint address".to_string(),
        }.into_response(),
    };

    let owner = match Pubkey::from_str(&req.owner) {
        Ok(p) => p,
        Err(_) => return ApiResponse::<()>::Error {
            success: false,
            error: "Invalid owner address".to_string(),
        }.into_response(),
    };

    let ix = match token_transfer(&spl_token::id(), &owner, &destination, &owner, &[], req.amount) {
        Ok(instr) => instr,
        Err(e) => return ApiResponse::<()>::Error {
            success: false,
            error: format!("Failed to create transfer instruction: {}", e),
        }.into_response(),
    };

    ApiResponse::Success {
        success: true,
        data: serde_json::json!({
            "program_id": ix.program_id.to_string(),
            "accounts": ix.accounts.iter().map(|a| serde_json::json!({
                "pubkey": a.pubkey.to_string(),
                "isSigner": a.is_signer,
            })).collect::<Vec<_>>(),
            "instruction_data": general_purpose::STANDARD.encode(ix.data),
        }),
    }.into_response()
}
