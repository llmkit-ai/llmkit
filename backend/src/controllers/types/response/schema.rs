use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateSchemaResponse {
    pub valid: bool,
    pub errors: Option<Vec<String>>,
}