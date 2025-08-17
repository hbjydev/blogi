use axum::{extract::{Query, State}, response::IntoResponse, Json};
use blogi_errors::{BlogiError, Result};
use blogi_lexicons::moe::hayden::blogi::actor::get_profiles;
use blogi_lexicons::moe::hayden::blogi::actor::get_profile;
use http::StatusCode;

use crate::state::AppState;

#[axum::debug_handler]
pub async fn list_actors(
    State(AppState { db, .. }): State<AppState>,
    Query(params): Query<get_profiles::ParametersData>,
) -> Result<impl IntoResponse> {
    let actors = db.list_actors(params.actors).await?;

    let output = get_profiles::OutputData {
        profiles: actors.into_iter().map(|actor| actor.into()).collect(),
    };

    Ok((StatusCode::OK, Json(output)))
}

#[axum::debug_handler]
pub async fn get_actor(
    State(AppState { db, .. }): State<AppState>,
    Query(params): Query<get_profile::ParametersData>,
) -> Result<impl IntoResponse> {
    let actor = db.get_actor(params.actor).await?;
    match actor {
        Some(actor) => Ok((StatusCode::OK, Json(actor))),
        None => Err(BlogiError::NotFound)
    }
}
