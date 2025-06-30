use crate::routes::{BalanceResponse, Message, Response, TokenCreateRequest, TokenMintRequest, MessageSignRequest, MessageVerifyRequest, SendSolRequest, SendTokenRequest};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::receive_message,
        crate::routes::get_balance,
        crate::routes::generate_keypair,
        crate::routes::create_token,
        crate::routes::mint_token,
        crate::routes::sign_message,
        crate::routes::verify_message,
        crate::routes::send_sol,
        crate::routes::send_token
    ),
    components(schemas(Message, Response, BalanceResponse, TokenCreateRequest, TokenMintRequest, MessageSignRequest, MessageVerifyRequest, SendSolRequest, SendTokenRequest)),
    tags((name = "Solana API", description = "Solana balance and token endpoints"))
)]
pub struct ApiDoc;
