use serde::Serialize;

use crate::db::models::Model;


#[derive(Debug, Serialize)]
pub struct ModelResponse {
    pub id: i64,
    pub provider: String,
    pub model: String
}


impl From<Model> for ModelResponse {
    fn from(model: Model) -> Self {
        ModelResponse {
            id: model.id,
            model: model.model_name,
            provider: model.provider,
        }
    }
}

