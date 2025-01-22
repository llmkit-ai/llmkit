use std::sync::Arc;
use anyhow::Result;
use sqlx::FromRow;

#[derive(Clone, Debug)]
pub struct ModelRepository {
    pool: Arc<sqlx::SqlitePool>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Model {
    pub id: i64,
    pub provider: String,
    pub model_name: String,
}

impl ModelRepository {
    pub async fn new(pool: Arc<sqlx::SqlitePool>) -> Result<Self> {
        Ok(ModelRepository { pool })
    }

    // Get a single model by ID
    pub async fn get_model_by_id(&self, id: i64) -> Result<Option<Model>> {
        let model = sqlx::query_as!(
            Model,
            r#"
            SELECT 
                id,
                provider,
                model_name
            FROM models
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(model)
    }

    // List all prompts with model info, ordered by creation time
    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let models = sqlx::query_as!(
            Model,
            r#"
                SELECT 
                    id,
                    provider,
                    model_name
                FROM models
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(models)
    }
}
