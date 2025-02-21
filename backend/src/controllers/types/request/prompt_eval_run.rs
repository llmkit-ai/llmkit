use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEvalRunRequest {
    pub id: i64,
    pub score: i64
}
