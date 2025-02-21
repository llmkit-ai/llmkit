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
