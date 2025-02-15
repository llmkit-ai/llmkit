use crate::db::types::prompt_sample::PromptSample;

use anyhow::Result;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct PromptSampleRepository {
    pool: sqlx::SqlitePool,
}

impl PromptSampleRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptSampleRepository { pool })
    }

    pub async fn get_by_id(&self, id: i64) -> Result<PromptSample> {
        sqlx::query_as!(
            PromptSample,
            r#"
            SELECT * 
            FROM prompt_sample 
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_by_prompt(&self, prompt_id: i64) -> Result<Vec<PromptSample>> {
        sqlx::query_as!(
            PromptSample,
            r#"
            SELECT * 
            FROM prompt_sample 
            WHERE prompt_id = ?
            "#,
            prompt_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(
        &self,
        prompt_id: i64,
        input_data: sqlx::types::Json<Value>,
        name: Option<String>,
    ) -> Result<PromptSample> {
        sqlx::query_as!(
            PromptSample,
            r#"
            INSERT INTO prompt_sample (prompt_id, input_data, name)
            VALUES (?, ?, ?)
            RETURNING *
            "#,
            prompt_id,
            input_data,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update(&self, id: i64, input_data: String, name: String) -> Result<PromptSample> {
        sqlx::query_as!(
            PromptSample,
            r#"
            UPDATE prompt_sample
            SET 
                input_data = ?,
                name = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            RETURNING *
            "#,
            input_data,
            name,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM prompt_sample
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
