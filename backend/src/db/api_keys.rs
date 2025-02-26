use anyhow::Result;
use rand::{distr::Alphanumeric, Rng};
use ring::pbkdf2;
use std::num::NonZeroU32;

use crate::db::types::api_key::ApiKeyRow;

const API_KEY_PREFIX: &str = "llmkit_";
static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = 32;
const SALT: &[u8] = b"llmkitsalt"; // In a real app, use a unique salt per key
const PBKDF2_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };

type Credential = [u8; CREDENTIAL_LEN];

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

    fn hash_api_key(api_key: &str) -> String {
        let mut credential = [0u8; CREDENTIAL_LEN];
        
        pbkdf2::derive(
            PBKDF2_ALG,
            PBKDF2_ITERATIONS,
            SALT,
            api_key.as_bytes(),
            &mut credential,
        );
        
        hex::encode(credential)
    }

    pub fn verify_api_key(provided_key: &str, stored_hash: &str) -> bool {
        let decoded_hash = match hex::decode(stored_hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        
        let credential: Credential = match decoded_hash.try_into() {
            Ok(c) => c,
            Err(_) => return false,
        };
        
        pbkdf2::verify(
            PBKDF2_ALG,
            PBKDF2_ITERATIONS,
            SALT,
            provided_key.as_bytes(),
            &credential,
        )
        .is_ok()
    }

    pub async fn create_api_key(&self, name: &str) -> Result<(i64, String)> {
        let key = Self::generate_api_key();
        let key_hash = Self::hash_api_key(&key);
        
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

    pub async fn verify_any_api_key(&self, api_key: &str) -> Result<bool> {
        let keys = sqlx::query!(
            r#"
            SELECT key_hash FROM api_key
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        for key in keys {
            if Self::verify_api_key(api_key, &key.key_hash) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
