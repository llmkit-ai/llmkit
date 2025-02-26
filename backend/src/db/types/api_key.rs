use sqlx::prelude::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyRow {
    pub id: i64,
    pub name: String,
    pub key_hash: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}