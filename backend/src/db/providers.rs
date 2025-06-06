use anyhow::Result;
use crate::db::types::providers::ProviderRow;

#[derive(Clone, Debug)]
pub struct ProviderRepository {
    pool: sqlx::SqlitePool,
}

impl ProviderRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(ProviderRepository { pool })
    }

    pub async fn get_provider_by_id(&self, id: i64) -> Result<Option<ProviderRow>> {
        let provider = sqlx::query_as!(
            ProviderRow,
            r#"
            SELECT 
                id,
                name,
                base_url,
                created_at
            FROM provider
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(provider)
    }

    pub async fn list_providers(&self) -> Result<Vec<ProviderRow>> {
        let providers = sqlx::query_as!(
            ProviderRow,
            r#"
            SELECT 
                id,
                name,
                base_url,
                created_at
            FROM provider
            ORDER BY name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(providers)
    }

    pub async fn get_provider_by_name(&self, name: &str) -> Result<Option<ProviderRow>> {
        let provider = sqlx::query_as!(
            ProviderRow,
            r#"
            SELECT 
                id,
                name,
                base_url,
                created_at
            FROM provider
            WHERE name = ?
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(provider)
    }

    pub async fn update_provider(&self, id: i32, base_url: Option<String>) -> Result<ProviderRow> {
        let provider = sqlx::query_as!(
            ProviderRow,
            r#"
            UPDATE provider
            SET base_url = ?
            WHERE id = ?
            RETURNING id, name, base_url, created_at
            "#,
            base_url,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(provider)
    }
}
