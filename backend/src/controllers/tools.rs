use axum::{
    extract::{Path, State},
    Json,
};

use crate::AppError;
use crate::AppState;

use super::types::{
    request::tools::{AssociateToolPromptVersionRequest, CreateToolRequest, UpdateToolRequest},
    response::tools::{ToolResponse, ToolVersionResponse},
};

pub async fn create_tool(
    State(state): State<AppState>,
    Json(payload): Json<CreateToolRequest>,
) -> Result<Json<ToolResponse>, AppError> {
    let tool_id = state
        .db
        .tool
        .create_tool(
            &payload.name,
            &payload.tool_name,
            &payload.description,
            &payload.parameters,
            payload.strict,
        )
        .await?;

    let tool = state
        .db
        .tool
        .get_tool(tool_id)
        .await
        .map_err(|_| AppError::NotFound("Tool not found after creation".into()))?;

    Ok(Json(tool.into()))
}

pub async fn get_tool(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<ToolResponse>, AppError> {
    let tool = state.db.tool.get_tool(id).await?;
    Ok(Json(tool.into()))
}

pub async fn list_tools(
    State(state): State<AppState>,
) -> Result<Json<Vec<ToolResponse>>, AppError> {
    let tools = state.db.tool.list_tools().await?;
    Ok(Json(tools.into_iter().map(|t| t.into()).collect()))
}

pub async fn update_tool(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateToolRequest>,
) -> Result<Json<ToolResponse>, AppError> {
    let updated = state
        .db
        .tool
        .update_tool(
            id,
            &payload.name,
            &payload.tool_name,
            &payload.description,
            &payload.parameters,
            payload.strict,
        )
        .await?;

    if !updated {
        return Err(AppError::NotFound("Tool not found".into()));
    }

    let tool = state
        .db
        .tool
        .get_tool(id)
        .await
        .map_err(|_| AppError::NotFound("Tool not found after update".into()))?;

    Ok(Json(tool.into()))
}

pub async fn delete_tool(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<(), AppError> {
    let deleted = state.db.tool.delete_tool(id).await?;

    if !deleted {
        return Err(AppError::NotFound("Tool not found".into()));
    }

    Ok(())
}

pub async fn get_tool_versions(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ToolVersionResponse>>, AppError> {
    // First check if the tool exists
    let _tool = state.db.tool.get_tool(id).await?;
    
    let versions = state.db.tool.get_tool_versions(id).await?;
    Ok(Json(versions.into_iter().map(|v| v.into()).collect()))
}

pub async fn get_tool_version(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<ToolVersionResponse>, AppError> {
    let version = state.db.tool.get_tool_version(id).await?;
    Ok(Json(version.into()))
}

pub async fn associate_tool_version_with_prompt_version(
    State(state): State<AppState>,
    Json(payload): Json<AssociateToolPromptVersionRequest>,
) -> Result<(), AppError> {
    state
        .db
        .tool
        .associate_tool_version_with_prompt_version(payload.tool_version_id, payload.prompt_version_id)
        .await?;
    Ok(())
}

pub async fn remove_tool_version_prompt_version_association(
    State(state): State<AppState>,
    Json(payload): Json<AssociateToolPromptVersionRequest>,
) -> Result<(), AppError> {
    let removed = state
        .db
        .tool
        .remove_tool_version_prompt_version_association(payload.tool_version_id, payload.prompt_version_id)
        .await?;
    
    if !removed {
        return Err(AppError::NotFound("Association not found".into()));
    }
    
    Ok(())
}

pub async fn get_prompt_versions_by_tool_version(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<i64>>, AppError> {
    // First check if the tool version exists
    let _version = state.db.tool.get_tool_version(id).await?;
    
    let prompt_version_ids = state.db.tool.get_prompt_versions_by_tool_version(id).await?;
    Ok(Json(prompt_version_ids))
}

pub async fn get_tool_versions_by_prompt_version(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ToolVersionResponse>>, AppError> {
    // We should check if the prompt version exists, but we'll assume it does for now
    // We could add a method to get a prompt version by id in the PromptRepository if needed
    
    let tool_versions = state.db.tool.get_tool_versions_by_prompt_version(id).await?;
    Ok(Json(tool_versions.into_iter().map(|v| v.into()).collect()))
}