use axum::{
    extract::{FromRequest, Json, Request},
    http::StatusCode,
};
use serde::de::DeserializeOwned;

// We'll take a simpler approach: a wrapper around the existing Json extractor
// that maps the error status code to 400

pub async fn extract_json_with_error_status<T>(
    req: Request,
) -> Result<Json<T>, (StatusCode, axum::Json<serde_json::Value>)>
where
    T: DeserializeOwned,
{
    match Json::<T>::from_request(req, &()).await {
        Ok(json) => Ok(json),
        Err(_rejection) => {
            // Return a 400 Bad Request with a JSON error body
            // Make sure it matches the expected error format by tests
            Err((
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": "Invalid or missing field in JSON request body",
                    "data": null
                }))
            ))
        }
    }
}
