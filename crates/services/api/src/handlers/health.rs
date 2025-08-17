use axum::{extract::State, http::StatusCode, response::IntoResponse};
use blogi_errors::Result;

use crate::state::AppState;

pub async fn xrpc_health(
    State(AppState { db, .. }): State<AppState>,
) -> Result<impl IntoResponse> {
    db.ping().await?;
    Ok(StatusCode::OK)
}
