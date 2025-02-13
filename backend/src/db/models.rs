use anyhow::Result;
use crate::db::types::models::ModelProviderRow;

#[derive(Clone, Debug)]
pub struct ModelRepository {
    pool: sqlx::SqlitePool,
}

impl ModelRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(ModelRepository { pool })
    }

    pub async fn create_model(
        &self,
        provider_id: i64,
        name: &str,
        supports_json: bool,
        supports_tools: bool,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query!(
            r#"
            INSERT INTO model (
                provider_id,
                name,
                supports_json,
                supports_tools
            ) VALUES (?, ?, ?, ?)
            "#,
            provider_id,
            name,
            supports_json,
            supports_tools
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    pub async fn update_model(
        &self,
        id: i64,
        provider_id: i64,
        name: &str,
        supports_json: bool,
        supports_tools: bool,
    ) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE model
            SET 
                provider_id = ?,
                name = ?,
                supports_json = ?,
                supports_tools = ?
            WHERE id = ?
            "#,
            provider_id,
            name,
            supports_json,
            supports_tools,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    pub async fn get_model_by_id(&self, id: i64) -> Result<Option<ModelProviderRow>> {
        let model = sqlx::query_as!(
            ModelProviderRow,
            r#"
            SELECT 
                m.id,
                m.provider_id,
                m.name as model_name,
                p.name as provider_name,
                m.supports_json,
                m.supports_tools,
                p.base_url as provider_base_url,
                m.created_at
            FROM model m
            JOIN provider p ON m.provider_id = p.id
            WHERE m.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(model)
    }

    pub async fn list_models(&self) -> Result<Vec<ModelProviderRow>> {
        let models = sqlx::query_as!(
            ModelProviderRow,
            r#"
            SELECT 
                m.id,
                m.provider_id,
                m.name as model_name,
                p.name as provider_name,
                m.supports_json,
                m.supports_tools,
                p.base_url as provider_base_url,
                m.created_at
            FROM model m
            JOIN provider p ON m.provider_id = p.id
            ORDER BY m.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }

    pub async fn get_model_by_name(&self, name: &str) -> Result<Option<ModelProviderRow>> {
        let model = sqlx::query_as!(
            ModelProviderRow,
            r#"
            SELECT 
                m.id,
                m.provider_id,
                m.name as model_name,
                p.name as provider_name,
                m.supports_json,
                m.supports_tools,
                p.base_url as provider_base_url,
                m.created_at
            FROM model m
            INNER JOIN provider p ON m.provider_id = p.id
            WHERE m.name = ?
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(model)
    }
}
