use anyhow::Result;
use crate::db::types::prompt::PromptRowWithModel;

use super::types::prompt::PromptWithModel;

#[derive(Clone, Debug)]
pub struct PromptRepository {
    pool: sqlx::SqlitePool,
}

impl PromptRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptRepository { pool })
    }

    #[cfg(test)]
    pub async fn in_memory(pool: sqlx::SqlitePool) -> Result<Self> {
        Self::new(pool.clone()).await
    }

    pub async fn create_prompt(
        &self,
        key: &str,
        system_prompt: &str,
        user_prompt: &str,
        model_id: i64,
        max_tokens: i64,
        temperature: f64,
        json_mode: bool,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query!(
            r#"
            INSERT INTO prompt (
                key, 
                system, 
                user, 
                model_id,
                max_tokens,
                temperature,
                json_mode
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            key,
            system_prompt,
            user_prompt,
            model_id,
            max_tokens,
            temperature,
            json_mode
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    pub async fn get_prompt(&self, id: i64) -> Result<PromptWithModel> {
        let prompt = sqlx::query_as!(
            PromptRowWithModel,
            r#"
            SELECT
                p.id, 
                p.key, 
                p.system, 
                p.user, 
                p.model_id,
                p.max_tokens,
                p.temperature,
                p.json_mode,
                m.name as model_name,
                pr.name as provider_name,
                p.created_at, 
                p.updated_at
            FROM prompt p
            JOIN model m ON p.model_id = m.id
            JOIN provider pr ON m.provider_id = pr.id
            WHERE p.id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(prompt.into())
    }

    pub async fn list_prompts(&self) -> Result<Vec<PromptWithModel>> {
        let prompts = sqlx::query_as!(
            PromptRowWithModel,
            r#"
            SELECT
                p.id, 
                p.key, 
                p.system, 
                p.user, 
                p.model_id,
                p.max_tokens,
                p.temperature,
                p.json_mode,
                m.name as model_name,
                pr.name as provider_name,
                p.created_at, 
                p.updated_at
            FROM prompt p
            JOIN model m ON p.model_id = m.id
            JOIN provider pr ON m.provider_id = pr.id
            ORDER BY p.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(prompts.into_iter().map(|p| p.into()).collect())
    }

    pub async fn update_prompt(
        &self,
        id: i64,
        key: &str,
        system_prompt: &str,
        user_prompt: &str,
        model_id: i64,
        max_tokens: i64,
        temperature: f64,
        json_mode: bool,
    ) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE prompt
            SET 
                key = ?, 
                system = ?, 
                user = ?, 
                model_id = ?,
                max_tokens = ?,
                temperature = ?,
                json_mode = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            key,
            system_prompt,
            user_prompt,
            model_id,
            max_tokens,
            temperature,
            json_mode,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    pub async fn delete_prompt(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM prompt
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }
}

