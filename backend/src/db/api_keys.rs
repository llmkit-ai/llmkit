use anyhow::Result;
use rand::{distr::Alphanumeric, Rng};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, PasswordHash, PasswordVerifier
};

use crate::db::types::api_key::ApiKeyRow;

const API_KEY_PREFIX: &str = "llmkit_";

#[derive(Clone, Debug)]
pub struct ApiKeyRepository {
    pool: sqlx::SqlitePool,
}

impl ApiKeyRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(ApiKeyRepository { pool })
    }

    fn generate_api_key() -> String {
        let key_part: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(24)
            .map(char::from)
            .collect();
        
        format!("{}{}", API_KEY_PREFIX, key_part)
    }

    fn hash_api_key(api_key: &str) -> Result<String, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        // Hash the API key
        let api_key_hash = argon2
            .hash_password(api_key.as_bytes(), &salt)?
            .to_string();
        
        Ok(api_key_hash)
    }

    pub fn verify_api_key(provided_key: &str, stored_hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(stored_hash) {
            Ok(h) => h,
            Err(_) => return false
        };

        match Argon2::default().verify_password(provided_key.as_bytes(), &parsed_hash) {
            Ok(()) => true,
            Err(e) => {
                tracing::error!("Failed to verify API key | {}", e);
                false
            }
        }
    }

    pub async fn create_api_key(&self, name: &str) -> Result<(i64, String)> {
        let key = Self::generate_api_key();
        let key_hash = Self::hash_api_key(&key)
            .map_err(|e| anyhow::anyhow!("Failed to hash API key: {}", e))?;
        
        let id = sqlx::query!(
            r#"
            INSERT INTO api_key (name, key_hash, created_at, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
            name,
            key_hash,
        )
        .fetch_one(&self.pool)
        .await?
        .id;
        
        Ok((id, key))
    }

    pub async fn get_api_keys(&self) -> Result<Vec<ApiKeyRow>> {
        let keys = sqlx::query_as!(
            ApiKeyRow,
            r#"
            SELECT 
                id, 
                name,
                key_hash,
                created_at,
                updated_at
            FROM api_key
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(keys)
    }

    pub async fn delete_api_key(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM api_key WHERE id = ?
            "#, 
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }

    pub async fn find_api_key_by_key(&self, api_key: &str) -> Result<Option<ApiKeyRow>> {
        // We need to query and check each API key since we need to verify
        // the plaintext API key against the stored hash using Argon2
        let keys = sqlx::query_as!(
            ApiKeyRow,
            r#"
            SELECT 
                id, 
                name,
                key_hash,
                created_at,
                updated_at
            FROM api_key
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Find the first key that verifies
        for key in keys {
            if Self::verify_api_key(api_key, &key.key_hash) {
                return Ok(Some(key));
            }
        }
        
        Ok(None)
    }

    pub async fn verify_any_api_key(&self, api_key: &str) -> Result<bool> {
        let key = self.find_api_key_by_key(api_key).await?;
        Ok(key.is_some())
    }
}
