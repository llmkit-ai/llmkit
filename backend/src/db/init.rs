use anyhow::Result;
use sqlx::migrate::Migrator;

use std::str::FromStr;

use super::{
    logs::LogRepository, 
    prompts::PromptRepository,
    prompt_samples::PromptSampleRepository,
    prompt_version_evals::PromptVersionEvalRepository,
    providers::ProviderRepository,
    models::ModelRepository,
};


static MIGRATOR: Migrator = sqlx::migrate!();


#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DbData {
    pub prompt: PromptRepository,
    pub prompt_version_eval: PromptVersionEvalRepository,
    pub prompt_sample: PromptSampleRepository,
    pub provider: ProviderRepository,
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
        let prompt_version_eval = PromptVersionEvalRepository::new(pool.clone()).await?;
        let prompt_sample = PromptSampleRepository::new(pool.clone()).await?;
        let provider = ProviderRepository::new(pool.clone()).await?;
        let log = LogRepository::new(pool.clone()).await?;
        let model = ModelRepository::new(pool.clone()).await?;

        Ok(DbData {
            log,
            model,
            prompt,
            prompt_version_eval,
            prompt_sample,
            provider
        })
    }
}
