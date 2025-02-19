#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PromptEvalTest {
    pub id: i64,
    pub prompt_id: i64,
    pub input_data: String,
    pub name: String,
    pub evaluation_type: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
