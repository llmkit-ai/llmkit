use anyhow::Result;
use std::sync::Arc;

use super::{
    logs::LogRepository, 
    prompts::PromptRepository,
    models::ModelRepository
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DbInit {
    pub prompt: PromptRepository,
    pub log: LogRepository,
    pub model: ModelRepository,
}

impl DbInit {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = Arc::new(sqlx::SqlitePool::connect(&db_url).await?);

        let prompt = PromptRepository::new(pool.clone()).await?;
        let log = LogRepository::new(pool.clone()).await?;
        let model = ModelRepository::new(pool.clone()).await?;

        Ok(DbInit {
            prompt,
            log,
            model
        })
    }
}
