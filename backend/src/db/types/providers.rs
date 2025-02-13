use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ProviderRow {
    pub id: i64,
    pub name: String,
    pub base_url: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}
