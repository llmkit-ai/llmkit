use axum::{
    extract::State,
    Json,
};

use super::types::response::providers::ProviderResponse;
use crate::{AppError, AppState};

pub async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProviderResponse>>, AppError> {
    let providers = state.db.provider.list_providers().await?;
    Ok(Json(providers.into_iter().map(|p| p.into()).collect()))
}