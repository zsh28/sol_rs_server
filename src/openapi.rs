use crate::routes::{BalanceResponse, Message, Response};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::hello,
        crate::routes::receive_message,
        crate::routes::get_balance
    ),
    components(schemas(Message, Response, BalanceResponse)),
    tags((name = "Solana API", description = "Solana balance and echo service"))
)]
pub struct ApiDoc;
