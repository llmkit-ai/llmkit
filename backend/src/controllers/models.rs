use axum::{
    extract::State,
    Json,
};

use crate::{AppError, AppState};

use super::types::response::models::ModelResponse;


pub async fn list_models(
    State(state): State<AppState>,
) -> Result<Json<Vec<ModelResponse>>, AppError> {
    let prompts = state.db.model.list_models().await?;
    Ok(Json(prompts.into_iter().map(|p| p.into()).collect()))
}
