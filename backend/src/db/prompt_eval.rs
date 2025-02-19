use anyhow::Result;
use serde_json::Value;

use super::types::prompt_eval::PromptEvalTest;

#[derive(Clone, Debug)]
pub struct PromptEvalTestRepository {
    pool: sqlx::SqlitePool,
}

impl PromptEvalTestRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptEvalTestRepository { pool })
    }

    pub async fn get_by_id(&self, id: i64) -> Result<PromptEvalTest> {
        sqlx::query_as!(
            PromptEvalTest,
            r#"
            SELECT * 
            FROM prompt_eval 
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_by_prompt(&self, prompt_id: i64) -> Result<Vec<PromptEvalTest>> {
        sqlx::query_as!(
            PromptEvalTest,
            r#"
            SELECT * 
            FROM prompt_eval 
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
        evaluation_type: &str,
        name: Option<String>,
    ) -> Result<PromptEvalTest> {
        sqlx::query_as!(
            PromptEvalTest,
            r#"
            INSERT INTO prompt_eval (prompt_id, input_data, name, evaluation_type)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
            prompt_id,
            input_data,
            name,
            evaluation_type
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update(&self, id: i64, input_data: String, name: String) -> Result<PromptEvalTest> {
        sqlx::query_as!(
            PromptEvalTest,
            r#"
            UPDATE prompt_eval
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
            DELETE FROM prompt_eval
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
