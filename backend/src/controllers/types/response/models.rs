use serde::Serialize;

use crate::db::types::models::ModelProviderRow;

#[derive(Debug, Serialize)]
pub struct ModelResponse {
    pub id: i64,
    pub provider_id: i64,
    pub name: String,
    pub provider_name: String,
    pub provider_base_url: String,
    pub supports_json: bool,
    pub supports_json_schema: bool,
    pub supports_tools: bool,
}

impl From<ModelProviderRow> for ModelResponse {
    fn from(row: ModelProviderRow) -> Self {
        Self {
            id: row.id,
            provider_id: row.provider_id,
            name: row.model_name,
            provider_name: row.provider_name.into(),
            provider_base_url: row.provider_base_url,
            supports_json: row.supports_json,
            supports_json_schema: row.supports_json_schema,
            supports_tools: row.supports_tools,
        }
    }
}

