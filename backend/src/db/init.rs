use anyhow::Result;
use sqlx::migrate::Migrator;

use std::str::FromStr;

use super::{
    api_keys::ApiKeyRepository, logs::LogRepository, models::ModelRepository, prompt_eval::PromptEvalTestRepository, 
    prompt_eval_run::PromptEvalTestRunRepository, prompts::PromptRepository, providers::ProviderRepository
};


static MIGRATOR: Migrator = sqlx::migrate!();


#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct DbData {
    pub prompt: PromptRepository,
    pub prompt_eval_run: PromptEvalTestRunRepository,
    pub prompt_eval: PromptEvalTestRepository,
    pub provider: ProviderRepository,
    pub log: LogRepository,
    pub model: ModelRepository,
    pub api_key: ApiKeyRepository,
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
        let prompt_eval_run = PromptEvalTestRunRepository::new(pool.clone()).await?;
        let prompt_eval = PromptEvalTestRepository::new(pool.clone()).await?;
        let provider = ProviderRepository::new(pool.clone()).await?;
        let log = LogRepository::new(pool.clone()).await?;
        let model = ModelRepository::new(pool.clone()).await?;
        let api_key = ApiKeyRepository::new(pool.clone()).await?;
        let user = UserRepository::new(pool.clone()).await?;

        Ok(DbData {
            log,
            model,
            prompt,
            prompt_eval_run,
            prompt_eval,
            provider,
            api_key,
            user
        })
    }
}