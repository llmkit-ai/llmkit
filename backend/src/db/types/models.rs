#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ModelRow {
   pub id: i64,
   pub model_name: String, 
}
