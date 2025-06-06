use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProviderRequest {
    pub base_url: Option<String>,
}