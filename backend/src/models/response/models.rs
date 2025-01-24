use serde::Serialize;

use crate::db::models::models::ModelRow;


#[derive(Debug, Serialize)]
pub struct ModelResponse {
    pub id: i64,
    pub model: String
}


impl From<ModelRow> for ModelResponse {
    fn from(model: ModelRow) -> Self {
        ModelResponse {
            id: model.id,
            model: model.model_name.into(),
        }
    }
}

