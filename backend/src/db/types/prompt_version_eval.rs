#[derive(sqlx::FromRow, Debug)]
pub struct PromptVersionEval {
    pub id: i64,
    pub prompt_version_id: i64,
    pub evaluation_type: String,
    pub score: Option<i64>,
    pub output: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
