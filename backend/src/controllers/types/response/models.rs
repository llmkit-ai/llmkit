use serde::Serialize;

use crate::{common::types::models::ModelName, db::types::models::ModelRow};


#[derive(Debug, Serialize)]
pub struct ModelResponse {
    pub id: i64,
    pub model: String,
    pub provider: String,
}


impl From<ModelRow> for ModelResponse {
    fn from(model: ModelRow) -> Self {
        let model_name: ModelName = model.model_name.into();

        ModelResponse {
            id: model.id,
            model: model_name.clone().into(),
            provider: model_name.provider()
        }
    }
}

