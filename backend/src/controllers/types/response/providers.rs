use serde::Serialize;

use crate::db::types::providers::ProviderRow;

#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub id: i64,
    pub name: String,
    pub base_url: String,
}

impl From<ProviderRow> for ProviderResponse {
    fn from(row: ProviderRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            base_url: row.base_url,
        }
    }
}