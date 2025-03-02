use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{AppError, AppState};
use super::types::response::logs::{ApiLogCountResponse, ApiLogResponse};


pub async fn get_log(
    Path(trace_id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<ApiLogResponse>, AppError> {
    let trace = state.db.log.get_log_by_id(trace_id).await?
        .ok_or(AppError::NotFound("API trace not found".into()))?;
    Ok(Json(trace.into()))
}

#[derive(Deserialize)]
pub struct PaginationParams {
    page: i64,
    page_size: i64,
}

pub async fn list_logs(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<ApiLogResponse>>, AppError> {
    let page = params.page;
    let page_size = params.page_size;

    let traces = state.db.log.list_logs(page, page_size).await?;
    Ok(Json(traces.into_iter().map(|t| t.into()).collect()))
}

pub async fn get_logs_count(
    State(state): State<AppState>,
) -> Result<Json<ApiLogCountResponse>, AppError> {
    let count = state.db.log.get_logs_count().await?;
    Ok(Json(ApiLogCountResponse { count }))
}

pub async fn get_log_by_provider_id(
    Path(provider_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiLogResponse>, AppError> {
    let trace = state.db.log.get_log_by_provider_response_id(&provider_id).await?
        .ok_or(AppError::NotFound(format!("Log with provider ID '{}' not found", provider_id)))?;
    Ok(Json(trace.into()))
}
