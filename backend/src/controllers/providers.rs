use axum::{
    extract::{Path, State},
    Json,
};

use super::types::{
    request::providers::UpdateProviderRequest,
    response::providers::ProviderResponse,
};
use crate::{AppError, AppState};

pub async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProviderResponse>>, AppError> {
    let providers = state.db.provider.list_providers().await?;
    
    let provider_responses: Vec<ProviderResponse> = providers
        .into_iter()
        .map(|p| {
            let mut response: ProviderResponse = p.into();
            
            // Check if API key exists in environment variables and base_url is configured
            let api_key_exists = match response.name.as_str() {
                "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
                "openrouter" => std::env::var("OPENROUTER_API_KEY").is_ok(),
                "azure" => std::env::var("AZURE_API_KEY").is_ok(),
                _ => false,
            };
            
            response.is_available = api_key_exists && response.base_url.is_some();
            response
        })
        .collect();
    
    Ok(Json(provider_responses))
}

pub async fn update_provider(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateProviderRequest>,
) -> Result<Json<ProviderResponse>, AppError> {
    let provider = state.db.provider.update_provider(id, payload.base_url).await?;
    Ok(Json(provider.into()))
}