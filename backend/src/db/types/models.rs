#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ModelProviderRow {
    pub id: i64,
    pub provider_id: i64, 
    pub model_name: String, 
    pub supports_json: bool, 
    pub supports_tools: bool, 
    pub provider_name: String, 
    pub provider_base_url: String, 
    pub created_at: chrono::NaiveDateTime
}
