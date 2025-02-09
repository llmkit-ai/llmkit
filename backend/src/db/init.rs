use anyhow::Result;
use sqlx::migrate::Migrator;

use std::str::FromStr;

use super::{
    logs::LogRepository, 
    prompts::PromptRepository,
    models::ModelRepository
};


static MIGRATOR: Migrator = sqlx::migrate!();


#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DbData {
    pub prompt: PromptRepository,
    pub log: LogRepository,
    pub model: ModelRepository,
}

impl DbData {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(db_url)?
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        ).await?;

        MIGRATOR.run(&pool).await?;

        let prompt = PromptRepository::new(pool.clone()).await?;
        let log = LogRepository::new(pool.clone()).await?;
        let model = ModelRepository::new(pool.clone()).await?;

        Ok(DbData {
            prompt,
            log,
            model
        })
    }
}
