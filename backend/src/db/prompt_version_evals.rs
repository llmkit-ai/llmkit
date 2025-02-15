use anyhow::Result;

use crate::db::types::prompt_version_eval::PromptVersionEval;

#[derive(Clone, Debug)]
pub struct PromptVersionEvalRepository {
    pool: sqlx::SqlitePool,
}

impl PromptVersionEvalRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptVersionEvalRepository { pool })
    }

    pub async fn get_by_id(&self, id: i64) -> Result<PromptVersionEval> {
        sqlx::query_as!(
            PromptVersionEval,
            r#"
            SELECT * 
            FROM prompt_version_eval 
            WHERE id = ?
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_by_prompt_version(
        &self,
        prompt_version_id: i64,
    ) -> Result<Vec<PromptVersionEval>> {
        sqlx::query_as!(
            PromptVersionEval,
            r#"
            SELECT * 
            FROM prompt_version_eval 
            WHERE prompt_version_id = ?
            "#,
            prompt_version_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn create(
        &self,
        prompt_version_id: i64,
        evaluation_type: String,
        score: Option<i64>,
    ) -> Result<PromptVersionEval> {
        sqlx::query_as!(
            PromptVersionEval,
            r#"
            INSERT INTO prompt_version_eval (prompt_version_id, evaluation_type, score)
            VALUES (?, ?, ?)
            RETURNING *
            "#,
            prompt_version_id,
            evaluation_type,
            score
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update(&self, eval: &PromptVersionEval) -> Result<PromptVersionEval> {
        sqlx::query_as!(
            PromptVersionEval,
            r#"
            UPDATE prompt_version_eval
            SET 
                prompt_version_id = ?,
                evaluation_type = ?,
                score = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            RETURNING *
            "#,
            eval.prompt_version_id,
            eval.evaluation_type,
            eval.score,
            eval.id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM prompt_version_eval
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
