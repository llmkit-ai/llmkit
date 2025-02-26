use anyhow::Result;
use ring::{digest, pbkdf2};
use std::num::NonZeroU32;

use crate::db::types::user::{UserResponse, UserRow};

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = 32;
const SALT: &[u8] = b"llmkitsalt"; // In a real app, use a unique salt per user
const PBKDF2_ITERATIONS: NonZeroU32 = unsafe { NonZeroU32::new_unchecked(100_000) };

type Credential = [u8; CREDENTIAL_LEN];

#[derive(Clone, Debug)]
pub struct UserRepository {
    pool: sqlx::SqlitePool,
}

impl UserRepository {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self> {
        Ok(UserRepository { pool })
    }

    fn hash_password(password: &str) -> String {
        let mut credential = [0u8; CREDENTIAL_LEN];
        
        pbkdf2::derive(
            PBKDF2_ALG,
            PBKDF2_ITERATIONS,
            SALT,
            password.as_bytes(),
            &mut credential,
        );
        
        hex::encode(credential)
    }

    fn verify_password(provided_password: &str, stored_hash: &str) -> bool {
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
            provided_password.as_bytes(),
            &credential,
        )
        .is_ok()
    }

    pub async fn create_user(&self, username: &str, password: &str, name: &str) -> Result<i64> {
        let password_hash = Self::hash_password(password);
        
        let id = sqlx::query!(
            r#"
            INSERT INTO user (username, password_hash, name, created_at, updated_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
            username,
            password_hash,
            name,
        )
        .fetch_one(&self.pool)
        .await?
        .id;
        
        Ok(id)
    }

    pub async fn get_users(&self) -> Result<Vec<UserResponse>> {
        let users = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, 
                username,
                password_hash,
                name,
                created_at,
                updated_at
            FROM user
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, 
                username,
                password_hash,
                name,
                created_at,
                updated_at
            FROM user
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user.map(UserResponse::from))
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<Option<UserResponse>> {
        let user = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, 
                username,
                password_hash,
                name,
                created_at,
                updated_at
            FROM user
            WHERE username = ?
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match user {
            Some(user) if Self::verify_password(password, &user.password_hash) => {
                Ok(Some(UserResponse::from(user)))
            },
            _ => Ok(None),
        }
    }

    pub async fn update_user(&self, id: i64, name: Option<&str>, password: Option<&str>) -> Result<bool> {
        let mut tx = self.pool.begin().await?;
        
        let current_user = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, 
                username,
                password_hash,
                name,
                created_at,
                updated_at
            FROM user
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&mut *tx)
        .await?;
        
        let Some(current_user) = current_user else {
            return Ok(false);
        };
        
        let new_name = name.unwrap_or(&current_user.name);
        let new_password_hash = if let Some(password) = password {
            Self::hash_password(password)
        } else {
            current_user.password_hash
        };
        
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user 
            SET name = ?, password_hash = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            new_name,
            new_password_hash,
            id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        
        tx.commit().await?;
        
        Ok(rows_affected > 0)
    }

    pub async fn delete_user(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM user WHERE id = ?
            "#, 
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }
}