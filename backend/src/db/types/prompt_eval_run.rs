#[derive(sqlx::FromRow, Debug)]
pub struct PromptEvalRun {
    pub id: i64,
    pub run_id: String,
    pub prompt_version_id: i64,
    pub prompt_eval_id: i64,
    pub prompt_eval_name: String,
    pub score: Option<i64>,
    pub output: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PromptEvalVersionPerformance {
    pub version_id: i64,
    pub version_number: i64,
    pub version_date: chrono::NaiveDateTime,
    pub avg_score: Option<f64>,
    pub run_count: i64,
}

