use std::sync::Arc;
use anyhow::Result;

use crate::db::models::prompt::PromptRowWithModel;

use super::models::prompt::PromptWithModel;

#[derive(Clone, Debug)]
pub struct PromptRepository {
    pool: Arc<sqlx::SqlitePool>,
}

impl PromptRepository {
    pub async fn new(pool: Arc<sqlx::SqlitePool>) -> Result<Self> {
        Ok(PromptRepository { pool })
    }

    // Create a new prompt
    pub async fn create_prompt(&self, key: &str, prompt: &str, model_id: i64) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query!(
            r#"
            INSERT INTO llm_prompts (key, prompt, model_id)
            VALUES (?, ?, ?)
            "#,
            key,
            prompt,
            model_id
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    // Get a single prompt by ID with model info
    pub async fn get_prompt(&self, id: i64) -> Result<PromptWithModel> {
        let prompt = sqlx::query_as!(
            PromptRowWithModel,
            r#"
            SELECT 
                p.id, p.key, p.prompt, p.model_id,
                m.model_name,
                p.created_at, p.updated_at
            FROM llm_prompts p
            JOIN models m ON p.model_id = m.id
            WHERE p.id = ?
            "#,
            id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(prompt.into())
    }

    // List all prompts with model info, ordered by creation time
    pub async fn list_prompts(&self) -> Result<Vec<PromptWithModel>> {
        let prompts = sqlx::query_as!(
            PromptRowWithModel,
            r#"
            SELECT 
                p.id, p.key, p.prompt, p.model_id,
                m.model_name,
                p.created_at, p.updated_at
            FROM llm_prompts p
            JOIN models m ON p.model_id = m.id
            ORDER BY p.created_at
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(prompts.into_iter().map(|p| p.into()).collect())
    }

    // Update an existing prompt
    pub async fn update_prompt(&self, id: i64, key: &str, prompt: &str, model_id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE llm_prompts
            SET key = ?, prompt = ?, model_id = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            key,
            prompt,
            model_id,
            id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }

    // Delete a prompt
    pub async fn delete_prompt(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM llm_prompts
            WHERE id = ?
            "#,
            id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }
}
