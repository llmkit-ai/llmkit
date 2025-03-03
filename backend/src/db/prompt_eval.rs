use anyhow::Result;
use serde_json::Value;

use super::types::prompt_eval::PromptEval;

#[derive(Clone, Debug)]
pub struct PromptEvalTestRepository {
    pool: sqlx::SqlitePool,
}

impl PromptEvalTestRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptEvalTestRepository { pool })
    }

    pub async fn get_by_id(&self, id: i64) -> Result<PromptEval> {
        sqlx::query_as!(
            PromptEval,
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

    pub async fn get_by_prompt(&self, prompt_id: i64) -> Result<Vec<PromptEval>> {
        sqlx::query_as!(
            PromptEval,
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
        system_prompt_input: Option<String>,
        user_prompt_input: String,
        evaluation_type: &str,
        name: Option<String>,
    ) -> Result<PromptEval> {
        sqlx::query_as!(
            PromptEval,
            r#"
            INSERT INTO prompt_eval (prompt_id, system_prompt_input, user_prompt_input, name, evaluation_type)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
            prompt_id,
            system_prompt_input,
            user_prompt_input,
            name,
            evaluation_type
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update(&self, id: i64, system_prompt_input: Option<String>, user_prompt_input: String, name: String) -> Result<PromptEval> {
        sqlx::query_as!(
            PromptEval,
            r#"
            UPDATE prompt_eval
            SET 
                system_prompt_input = ?,
                user_prompt_input = ?,
                name = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            RETURNING *
            "#,
            system_prompt_input,
            user_prompt_input,
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
