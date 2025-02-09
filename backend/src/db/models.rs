use anyhow::Result;

use crate::db::types::models::ModelRow;

#[derive(Clone, Debug)]
pub struct ModelRepository {
    pool: sqlx::SqlitePool,
}

impl ModelRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(ModelRepository { pool })
    }

    // Get a single model by ID
    pub async fn get_model_by_id(&self, id: i64) -> Result<Option<ModelRow>> {
        let model = sqlx::query_as!(
            ModelRow,
            r#"
            SELECT 
                id,
                model_name
            FROM model
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(model)
    }

    // List all prompts with model info, ordered by creation time
    pub async fn list_models(&self) -> Result<Vec<ModelRow>> {
        let models = sqlx::query_as!(
            ModelRow,
            r#"
                SELECT 
                    id,
                    model_name
                FROM model
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(models)
    }
}
