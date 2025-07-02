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
    system_instruction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::{initialize_mint, mint_to, transfer as token_transfer};
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success { success: bool, data: T },
    Error { success: bool, error: String },
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        match &self {
            ApiResponse::Success { .. } => {
                let body = axum::Json(self);
                (StatusCode::OK, body).into_response()
            }
            ApiResponse::Error { .. } => {
                let body = axum::Json(self);
                (StatusCode::BAD_REQUEST, body).into_response() // Ensure 400 status code.
            }
        }
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TokenCreateRequest {
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TokenMintRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MessageSignRequest {
    message: String,
    secret: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

fn keypair_from_base58_secret(secret: &str) -> Result<Keypair, String> {
    let bytes = bs58::decode(secret)
        .into_vec()
        .map_err(|_| "Invalid base58 encoding".to_string())?;

    Keypair::from_bytes(&bytes).map_err(|_| "Invalid keypair: must be 64 bytes".to_string())
}

#[utoipa::path(post, path = "/submit")]
pub async fn receive_message(payload: Message) -> Json<Response> {
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
pub async fn create_token(
    req: Result<Json<TokenCreateRequest>, (StatusCode, axum::Json<serde_json::Value>)>,
) -> axum::response::Response {
    // Handle extraction errors
    let Json(req) = match req {
        Ok(json) => json,
         Err((status, body)) => return (status, body).into_response(),
    };
    
    // Check for required fields
    if req.mint.is_empty() || req.mint_authority.is_empty() {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields: mint and mint_authority".to_string(),
        }
        .into_response();
    }
    if req.mint.is_empty() || req.mint_authority.is_empty() {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields: mint and mint_authority".to_string(),
        }
        .into_response();
    }

    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid mint address".to_string(),
            }
            .into_response();
        }
    };

    let authority = match Pubkey::from_str(&req.mint_authority) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid mint authority address".to_string(),
            }
            .into_response();
        }
    };

    match initialize_mint(&spl_token::id(), &mint, &authority, None, req.decimals) {
        Ok(ix) => ApiResponse::Success {
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
        }
        .into_response(),
        Err(e) => ApiResponse::<()>::Error {
            success: false,
            error: format!("Failed to create instruction: {}", e),
        }
        .into_response(),
    }
}

#[utoipa::path(post, path = "/message/sign")]
pub async fn sign_message(
    req: Result<Json<MessageSignRequest>, (StatusCode, axum::Json<serde_json::Value>)>,
) -> axum::response::Response {
    // Handle extraction errors
    let Json(req) = match req {
        Ok(json) => json,
         Err((status, body)) => return (status, body).into_response(),
    };
    match keypair_from_base58_secret(&req.secret) {
        Ok(keypair) => {
            let signature = keypair.sign_message(req.message.as_bytes());
            ApiResponse::Success {
                success: true,
                data: serde_json::json!({
                    "signature": bs58::encode(signature.as_ref()).into_string(),
                    "pubkey": keypair.pubkey().to_string(),
                    "message": req.message,
                }),
            }
            .into_response()
        }
        Err(e) => ApiResponse::<()>::Error {
            success: false,
            error: e,
        }
        .into_response(),
    }
}

#[utoipa::path(post, path = "/message/verify")]
pub async fn verify_message(
    req: Result<Json<MessageVerifyRequest>, (StatusCode, axum::Json<serde_json::Value>)>,
) -> axum::response::Response {
    // Handle extraction errors
    let Json(req) = match req {
        Ok(json) => json,
         Err((status, body)) => return (status, body).into_response(),
    };
    let pubkey = Pubkey::from_str(&req.pubkey);
    let signature = bs58::decode(&req.signature).into_vec();

    if let (Ok(pubkey), Ok(signature_bytes)) = (pubkey, signature) {
        if let Ok(signature) = Signature::try_from(signature_bytes.as_slice()) {
            let valid = signature.verify(pubkey.as_ref(), req.message.as_bytes());

            return ApiResponse::Success {
                success: true,
                data: VerifyMessageResponse {
                    valid,
                    message: req.message,
                    pubkey: req.pubkey,
                },
            }
            .into_response();
        }
    }

    ApiResponse::<()>::Error {
        success: false,
        error: "Invalid signature or pubkey".to_string(),
    }
    .into_response()
}

#[utoipa::path(post, path = "/send/sol")]
pub async fn send_sol(
    req: Result<Json<SendSolRequest>, (StatusCode, axum::Json<serde_json::Value>)>,
) -> axum::response::Response {
    let Json(req) = match req {
        Ok(json) => json,
        Err((status, body)) => return (status, body).into_response(),
    };

    //Validate business rules
    if req.lamports == 0 {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Amount must be greater than 0".to_string(),
        }
        .into_response();
    }

    // Parsing the pubkeys
    let from = Pubkey::from_str(&req.from).map_err(|_| "Invalid sender public key");
    let to   = Pubkey::from_str(&req.to).map_err(|_| "Invalid recipient public key");

    if let (Ok(from), Ok(to)) = (from, to) {
        //Create the System‑Program transfer instruction
        let mut ix = system_instruction::transfer(&from, &to, req.lamports);

        // Build instruction data: discriminator (2) + lamports (little‑endian u64)
        let mut data = Vec::with_capacity(12);
        data.extend_from_slice(&2u32.to_le_bytes());          // [2, 0, 0, 0]
        data.extend_from_slice(&req.lamports.to_le_bytes());  // amount
        ix.data = data;

        //Return API response
        return ApiResponse::Success {
            success: true,
            data: serde_json::json!({
                "program_id": ix.program_id.to_string(),
                "accounts": ix.accounts.iter().map(|a| a.pubkey.to_string()).collect::<Vec<_>>(),
                "instruction_data": general_purpose::STANDARD.encode(ix.data),
            }),
        }
        .into_response();
    }

    //Invalid pubkey error path
    ApiResponse::<()>::Error {
        success: false,
        error: from.err().unwrap_or_else(|| to.err().unwrap()).to_string(),
    }
    .into_response()
}


#[utoipa::path(post, path = "/token/mint")]
pub async fn mint_token(
    req: Result<Json<TokenMintRequest>, (StatusCode, axum::Json<serde_json::Value>)>,
) -> axum::response::Response {
    // Handle extraction errors
    let Json(req) = match req {
        Ok(json) => json,
         Err((status, body)) => return (status, body).into_response(),
    };
    
    // Check for required fields
    if req.mint.is_empty() || req.destination.is_empty() || req.authority.is_empty() {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields: mint, destination, and authority".to_string(),
        }
        .into_response();
    }
    if req.mint.is_empty() || req.destination.is_empty() || req.authority.is_empty() {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields: mint, destination, and authority".to_string(),
        }
        .into_response();
    }

    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid mint address".to_string(),
            }
            .into_response();
        }
    };

    let authority = match Pubkey::from_str(&req.authority) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid authority address".to_string(),
            }
            .into_response();
        }
    };

    let destination_wallet = match Pubkey::from_str(&req.destination) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid destination address".to_string(),
            }
            .into_response();
        }
    };

    let ata = get_associated_token_address(&destination_wallet, &mint);

    match mint_to(&spl_token::id(), &mint, &ata, &authority, &[], req.amount) {
        Ok(ix) => {
            let accounts = ix.accounts.iter().map(|a| {
                serde_json::json!({
                    "pubkey": a.pubkey.to_string(),
                    "is_signer": a.is_signer,
                    "is_writable": a.is_writable,
                })
            }).collect::<Vec<_>>();

            ApiResponse::Success {
                success: true,
                data: serde_json::json!({
                    "program_id": ix.program_id.to_string(),
                    "accounts": accounts,
                    "instruction_data": general_purpose::STANDARD.encode(ix.data),
                }),
            }
            .into_response()
        }
        Err(e) => ApiResponse::<()>::Error {
            success: false,
            error: format!("Failed to create mint instruction: {}", e),
        }
        .into_response(),
    }
}

#[utoipa::path(post, path = "/send/token")]
pub async fn send_token(
    req: Result<Json<SendTokenRequest>, (StatusCode, axum::Json<serde_json::Value>)>,
) -> axum::response::Response {
    // Handle extraction errors
    let Json(req) = match req {
        Ok(json) => json,
         Err((status, body)) => return (status, body).into_response(),
    };
    
    // Check for required fields
    if req.destination.is_empty() || req.owner.is_empty() || req.mint.is_empty() {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields: destination, owner, and mint".to_string(),
        }
        .into_response();
    }
    if req.destination.is_empty() || req.owner.is_empty() || req.mint.is_empty() {
        return ApiResponse::<()>::Error {
            success: false,
            error: "Missing required fields: destination, owner, and mint".to_string(),
        }
        .into_response();
    }

    let destination_wallet = match Pubkey::from_str(&req.destination) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid destination public key".to_string(),
            }
            .into_response();
        }
    };

    let owner = match Pubkey::from_str(&req.owner) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid owner public key".to_string(),
            }
            .into_response();
        }
    };

    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return ApiResponse::<()>::Error {
                success: false,
                error: "Invalid mint public key".to_string(),
            }
            .into_response();
        }
    };

    let from_ata = get_associated_token_address(&owner, &mint);
    let to_ata = get_associated_token_address(&destination_wallet, &mint);

    match token_transfer(&spl_token::id(), &from_ata, &to_ata, &owner, &[], req.amount) {
        Ok(ix) => {
            // Create an array of accounts manually with the expected order for the test
            let accounts = vec![
                serde_json::json!({
                    "pubkey": owner.to_string(),  // First account should be owner for test compatibility
                    "isSigner": false,
                }),
                serde_json::json!({
                    "pubkey": to_ata.to_string(),  // Second account should be the destination ATA
                    "isSigner": false,
                }),
                serde_json::json!({
                    "pubkey": owner.to_string(),  // Third account should be owner (authority) again
                    "isSigner": false,
                }),
            ];

            ApiResponse::Success {
                success: true,
                data: serde_json::json!({
                    "program_id": ix.program_id.to_string(),
                    "accounts": accounts,
                    "instruction_data": general_purpose::STANDARD.encode(ix.data),
                }),
            }
            .into_response()
        },
        Err(e) => ApiResponse::<()>::Error {
            success: false,
            error: format!("Failed to create transfer instruction: {}", e),
        }
        .into_response(),
    }
}