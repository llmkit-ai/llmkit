use std::sync::Arc;
use anyhow::Result;
use sqlx::FromRow;

#[derive(Clone, Debug)]
pub struct LogRepository {
    pool: Arc<sqlx::SqlitePool>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LlmApiTrace {
    pub id: i64,
    pub prompt_id: Option<i64>,
    pub model_id: i64,
    pub request_data: String,
    pub response_data: Option<String>,
    pub status_code: Option<i64>,
    pub latency_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
}

impl LogRepository {
    pub async fn new(pool: Arc<sqlx::SqlitePool>) -> Result<Self> {
        Ok(LogRepository { pool })
    }

    // Create a new API trace
    pub async fn create_trace(
        &self,
        prompt_id: Option<i64>,
        model_id: i64,
        request_data: &str,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;
        let id = sqlx::query!(
            r#"
            INSERT INTO llm_api_traces (
                prompt_id,
                model_id,
                request_data
            ) VALUES (?, ?, ?)
            "#,
            prompt_id,
            model_id,
            request_data
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();
        Ok(id)
    }

    // Update trace with response data
    pub async fn update_trace_response(
        &self,
        id: i64,
        response_data: &str,
        status_code: i32,
        latency_ms: i32,
        input_tokens: i32,
        output_tokens: i32,
    ) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE llm_api_traces
            SET 
                response_data = ?,
                status_code = ?,
                latency_ms = ?,
                input_tokens = ?,
                output_tokens = ?
            WHERE id = ?
            "#,
            response_data,
            status_code,
            latency_ms,
            input_tokens,
            output_tokens,
            id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }

    // Update trace with error information
    pub async fn update_trace_error(
        &self,
        id: i64,
        error_code: &str,
        error_message: &str,
    ) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE llm_api_traces
            SET 
                error_code = ?,
                error_message = ?
            WHERE id = ?
            "#,
            error_code,
            error_message,
            id
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();
        Ok(rows_affected > 0)
    }

    // Get a single trace by ID
    pub async fn get_trace_by_id(&self, id: i64) -> Result<Option<LlmApiTrace>> {
        let trace = sqlx::query_as!(
            LlmApiTrace,
            r#"
            SELECT 
                id,
                prompt_id,
                model_id,
                request_data,
                response_data,
                status_code,
                latency_ms,
                input_tokens,
                output_tokens,
                error_code,
                error_message,
                created_at
            FROM llm_api_traces
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(trace)
    }

    // List all traces ordered by creation time
    pub async fn list_traces(&self) -> Result<Vec<LlmApiTrace>> {
        let traces = sqlx::query_as!(
            LlmApiTrace,
            r#"
            SELECT 
                id,
                prompt_id,
                model_id,
                request_data,
                response_data,
                status_code,
                latency_ms,
                input_tokens,
                output_tokens,
                error_code,
                error_message,
                created_at
            FROM llm_api_traces
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(traces)
    }

    // List traces by prompt ID
    pub async fn list_traces_by_prompt(&self, prompt_id: i64) -> Result<Vec<LlmApiTrace>> {
        let traces = sqlx::query_as!(
            LlmApiTrace,
            r#"
            SELECT 
                id,
                prompt_id,
                model_id,
                request_data,
                response_data,
                status_code,
                latency_ms,
                input_tokens,
                output_tokens,
                error_code,
                error_message,
                created_at
            FROM llm_api_traces
            WHERE prompt_id = ?
            ORDER BY created_at DESC
            "#,
            prompt_id
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(traces)
    }

    // Delete a trace
    pub async fn delete_trace(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM llm_api_traces
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
