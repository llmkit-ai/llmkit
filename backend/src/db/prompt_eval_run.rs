use anyhow::Result;

use super::types::prompt_eval_run::PromptEvalRun;

#[derive(Clone, Debug)]
pub struct PromptEvalTestRunRepository {
    pool: sqlx::SqlitePool,
}

impl PromptEvalTestRunRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(PromptEvalTestRunRepository { pool })
    }

    pub async fn get_by_id(&self, id: i64) -> Result<PromptEvalRun> {
        sqlx::query_as!(
            PromptEvalRun,
            r#"
            SELECT * 
            FROM prompt_eval_run 
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
    ) -> Result<Vec<PromptEvalRun>> {
        sqlx::query_as!(
            PromptEvalRun,
            r#"
            SELECT * 
            FROM prompt_eval_run 
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
        prompt_eval_id: i64,
        score: Option<i64>,
        output: &str
    ) -> Result<PromptEvalRun> {
        sqlx::query_as!(
            PromptEvalRun,
            r#"
            INSERT INTO prompt_eval_run (prompt_version_id, prompt_eval_id, score, output)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
            prompt_version_id,
            prompt_eval_id,
            score,
            output
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM prompt_eval_run
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
