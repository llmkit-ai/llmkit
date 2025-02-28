use anyhow::{Result, anyhow};
use sqlx::SqlitePool;

use crate::db::types::user::User;

#[derive(Clone, Debug)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        Ok(UserRepository { pool })
    }
    
    pub async fn check_registration_completed(&self) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            SELECT value FROM system_settings
            WHERE key = 'registration_completed'
            "#
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match result {
            Some(row) => Ok(row.value == "true"),
            None => Ok(false)
        }
    }
    
    pub async fn set_registration_completed(&self) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE system_settings
            SET value = 'true', updated_at = CURRENT_TIMESTAMP
            WHERE key = 'registration_completed'
            "#
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn create(&self, name: &str, email: &str, password_hash: &str) -> Result<i64> {
        // Check if registration is already completed
        if self.check_registration_completed().await? {
            return Err(anyhow!("Registration is closed. System already has a user account."));
        }
        
        let id = sqlx::query!(
            r#"
            INSERT INTO user (name, email, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
            name,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await?
        .id;
        
        // Mark registration as completed
        self.set_registration_completed().await?;

        Ok(id)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                name,
                email,
                password_hash,
                created_at,
                updated_at
            FROM user
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                name,
                email,
                password_hash,
                created_at,
                updated_at
            FROM user
            WHERE email = ?
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update(&self, id: i64, name: &str, email: &str, password_hash: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user
            SET name = ?,
                email = ?,
                password_hash = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            name,
            email,
            password_hash,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    pub async fn delete(&self, id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM user
            WHERE id = ?
            "#,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        // Also reset the registration_completed flag to allow new registration
        sqlx::query!(
            r#"
            UPDATE system_settings
            SET value = 'false', updated_at = CURRENT_TIMESTAMP
            WHERE key = 'registration_completed'
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(rows_affected > 0)
    }
    
    pub async fn find_all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                name,
                email,
                password_hash,
                created_at,
                updated_at
            FROM user
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }
}
