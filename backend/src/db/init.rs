use anyhow::Result;
use sqlx::migrate::Migrator;

use std::sync::Arc;

use super::{
    logs::LogRepository, 
    prompts::PromptRepository,
    llm_models::ModelRepository
};


static MIGRATOR: Migrator = sqlx::migrate!();


#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DbInit {
    pub prompt: PromptRepository,
    pub log: LogRepository,
    pub model: ModelRepository,
}

impl DbInit {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect(&db_url).await?;
        MIGRATOR.run(&pool).await?;

        let arc_pool = Arc::new(pool);

        let prompt = PromptRepository::new(arc_pool.clone()).await?;
        let log = LogRepository::new(arc_pool.clone()).await?;
        let model = ModelRepository::new(arc_pool.clone()).await?;

        Ok(DbInit {
            prompt,
            log,
            model
        })
    }
}
