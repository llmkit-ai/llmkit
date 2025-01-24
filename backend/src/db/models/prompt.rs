use sqlx::FromRow;

use super::models::ModelName;


#[derive(Debug, Clone, FromRow)]
pub struct PromptRow {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model_id: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct PromptRowWithModel {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model_id: i64,
    pub model_name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Into<PromptWithModel> for PromptRowWithModel {
    fn into(self) -> PromptWithModel {
        let model: ModelName = self.model_name.into();

        PromptWithModel {
            id: self.id,
            key: self.key,
            prompt: self.prompt,
            created_at: self.created_at,
            updated_at: self.updated_at,
            model_id: self.model_id,
            provider: model.provider(),
            model_name: model.into()
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PromptWithModel {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model_id: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub provider: String,
    pub model_name: String,
}
