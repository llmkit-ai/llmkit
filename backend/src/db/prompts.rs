use std::sync::Arc;
use anyhow::Result;
use sqlx::FromRow;

#[derive(Clone, Debug)]
pub struct PromptRepository {
    pool: Arc<sqlx::SqlitePool>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LlmPrompt {
    pub id: i64,
    pub key: String,
    pub prompt: String,
    pub model: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl PromptRepository {
    pub async fn new(pool: Arc<sqlx::SqlitePool>) -> Result<Self> {
        Ok(PromptRepository { pool })
    }

    // Create a new prompt
    pub async fn create_prompt(&self, key: &str, prompt: &str, model: &str) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query!(
            r#"
            INSERT INTO llm_prompts (key, prompt, model)
            VALUES (?, ?, ?)
            "#,
            key,
            prompt,
            model
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    // // Get a single prompt by ID
    pub async fn get_prompt(&self, id: i64) -> Result<Option<LlmPrompt>> {
        let prompt = sqlx::query_as!(
            LlmPrompt,
            r#"
            SELECT id, key, prompt, model, created_at, updated_at
            FROM llm_prompts
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(prompt)
    }

    // List all prompts ordered by creation time
    pub async fn list_prompts(&self) -> Result<Vec<LlmPrompt>> {
        let prompts = sqlx::query_as!(
            LlmPrompt,
            r#"
            SELECT id, key, prompt, model, created_at, updated_at
            FROM llm_prompts
            ORDER BY created_at
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(prompts)
    }

    // Update an existing prompt
    pub async fn update_prompt(&self, id: i64, key: &str, prompt: &str, model: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE llm_prompts
            SET key = ?, prompt = ?, model = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            key,
            prompt,
            model,
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
