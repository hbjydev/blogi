use axum::{extract::State, response::IntoResponse, Json};
use blogi_errors::Result;
use blogi_lexicons::moe::hayden::blogi::actor::get_profiles;
use http::StatusCode;

use crate::state::AppState;

#[axum::debug_handler]
pub async fn list_actors(
    State(AppState { db, .. }): State<AppState>,
) -> Result<impl IntoResponse> {
    let actors = db.list_actors().await?;

    let output = get_profiles::OutputData {
        profiles: actors.into_iter().map(|actor| actor.into()).collect(),
    };

    Ok((StatusCode::OK, Json(output)))
}
