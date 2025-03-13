use crate::common::types::models::LlmApiProvider;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ModelProviderRow {
    pub id: i64,
    pub provider_id: i64, 
    pub model_name: String, 
    pub supports_json: bool, 
    pub supports_json_schema: bool, 
    pub supports_tools: bool, 
    pub provider_name: LlmApiProvider, 
    pub provider_base_url: String, 
    pub created_at: chrono::NaiveDateTime
}
