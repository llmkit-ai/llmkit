use anyhow::Result;
use crate::db::types::log::{LogRow, LogRowModel};

#[derive(Clone, Debug)]
pub struct LogRepository {
    pool: sqlx::SqlitePool,
}

impl LogRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(LogRepository { pool })
    }

    #[cfg(test)]
    pub async fn in_memory(pool: sqlx::SqlitePool) -> Result<Self> {
        Self::new(pool.clone()).await
    }

    pub async fn create_log(
        &self,
        prompt_id: Option<i64>,
        model_id: i64,
        response_data: Option<&str>,
        status_code: Option<i64>,
        input_tokens: Option<i64>,
        output_tokens: Option<i64>,
        reasoning_tokens: Option<i64>,
        request_body: Option<&str>,
        provider_response_id: &str
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query!(
            r#"
            INSERT INTO log (
                prompt_id,
                model_id,
                response_data,
                status_code,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                request_body,
                provider_response_id,
                created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
            prompt_id,
            model_id,
            response_data,
            status_code,
            input_tokens,
            output_tokens,
            reasoning_tokens,
            request_body,
            provider_response_id,
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    pub async fn update_log_response(
        &self,
        id: i64,
        response_data: &str,
        status_code: i32,
        input_tokens: i32,
        output_tokens: i32,
        provider_response_id: &str,
    ) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE log
            SET 
                response_data = ?,
                status_code = ?,
                input_tokens = ?,
                output_tokens = ?,
                provider_response_id = ?
            WHERE id = ?
            "#,
            response_data,
            status_code,
            input_tokens,
            output_tokens,
            provider_response_id,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }

    pub async fn get_log_by_id(&self, id: i64) -> Result<Option<LogRowModel>> {
        let log = sqlx::query_as!(
            LogRowModel,
            r#"
            SELECT 
                l.id,
                l.prompt_id,
                l.model_id,
                m.name as model_name,
                p.name as provider_name,
                l.response_data,
                l.status_code,
                l.input_tokens,
                l.output_tokens,
                l.reasoning_tokens,
                l.created_at,
                l.request_body,
                l.provider_response_id
            FROM log l
            JOIN model m ON m.id = l.model_id
            JOIN provider p ON m.provider_id = p.id
            WHERE l.id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(log)
    }

    pub async fn list_logs(&self, page: i64, page_size: i64) -> Result<Vec<LogRowModel>> {
        let offset = (page - 1) * page_size;

        let logs = sqlx::query_as!(
            LogRowModel,
            r#"
                SELECT 
                    l.id,
                    l.prompt_id,
                    l.model_id,
                    m.name as model_name,
                    p.name as provider_name,
                    l.response_data,
                    l.status_code,
                    l.input_tokens,
                    l.output_tokens,
                    l.reasoning_tokens,
                    l.created_at,
                    l.request_body,
                    l.provider_response_id
                FROM log l
                INNER JOIN model m ON m.id = l.model_id
                INNER JOIN provider p ON m.provider_id = p.id
                ORDER BY l.created_at DESC
                LIMIT ? OFFSET ?
            "#,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn list_logs_by_prompt(&self, prompt_id: i64) -> Result<Vec<LogRow>> {
        let logs = sqlx::query_as!(
            LogRow,
            r#"
            SELECT 
                id,
                prompt_id,
                model_id,
                response_data,
                status_code,
                input_tokens,
                output_tokens,
                reasoning_tokens,
                created_at,
                request_body,
                provider_response_id
            FROM log
            WHERE prompt_id = ?
            ORDER BY created_at DESC
            "#,
            prompt_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(logs)
    }

    pub async fn get_logs_count(&self) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM log
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    pub async fn get_log_by_provider_response_id(&self, provider_response_id: &str) -> Result<Option<LogRowModel>> {
        let log = sqlx::query_as!(
            LogRowModel,
            r#"
            SELECT 
                l.id,
                l.prompt_id,
                l.model_id,
                m.name as model_name,
                p.name as provider_name,
                l.response_data,
                l.status_code,
                l.input_tokens,
                l.output_tokens,
                l.reasoning_tokens,
                l.created_at,
                l.request_body,
                l.provider_response_id
            FROM log l
            JOIN model m ON m.id = l.model_id
            JOIN provider p ON m.provider_id = p.id
            WHERE l.provider_response_id = ?
            "#,
            provider_response_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(log)
    }
}

