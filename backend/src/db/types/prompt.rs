use sqlx::FromRow;
use crate::common::types::models::LlmModel;

#[derive(Debug, Clone, FromRow)]
pub struct PromptRow {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct PromptRowWithModel {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub model_name: String,
    pub provider_name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct PromptWithModel {
    pub id: i64,
    pub key: String,
    pub system: String,
    pub user: String,
    pub model_id: i64,
    pub max_tokens: i64,
    pub temperature: f64,
    pub json_mode: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub provider: String,
    pub model_name: String,
}

impl Into<PromptWithModel> for PromptRowWithModel {
    fn into(self) -> PromptWithModel {
        let model: LlmModel = self.model_name.into();
        PromptWithModel {
            id: self.id,
            key: self.key,
            system: self.system,
            user: self.user,
            model_id: self.model_id,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            json_mode: self.json_mode,            
            created_at: self.created_at,
            updated_at: self.updated_at,
            provider: model.provider(),
            model_name: model.into()
        }
    }
}
