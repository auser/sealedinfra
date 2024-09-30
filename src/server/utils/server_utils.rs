use axum::http::StatusCode;
use serde_json::json;

use crate::error::SealedError;

pub fn handle_error(err: SealedError) -> (StatusCode, axum::Json<serde_json::Value>) {
    let msg = axum::Json(json!({ "error": format!("{}", &err) }));
    (StatusCode::INTERNAL_SERVER_ERROR, msg)
}
