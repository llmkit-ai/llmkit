use axum::{
    extract::{Path, State},
    Json,
};

use crate::{AppError, AppState};

use super::types::response::logs::ApiTraceResponse;


pub async fn get_api_trace(
    Path(trace_id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<ApiTraceResponse>, AppError> {
    let trace = state.db.log.get_trace_by_id(trace_id).await?
        .ok_or(AppError::NotFound("API trace not found".into()))?;
    Ok(Json(trace.into()))
}

pub async fn list_api_traces(
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiTraceResponse>>, AppError> {
    let traces = state.db.log.list_traces().await?;
    Ok(Json(traces.into_iter().map(|t| t.into()).collect()))
}
