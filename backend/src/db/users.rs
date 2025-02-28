use anyhow::Result;
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

    pub async fn create(&self, name: &str, email: &str, password_hash: &str, role: &str, status: &str) -> Result<i64> {
        let id = sqlx::query!(
            r#"
            INSERT INTO user (name, email, password_hash, role, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
            name,
            email,
            password_hash,
            role,
            status
        )
        .fetch_one(&self.pool)
        .await?
        .id;

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
                role,
                status,
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
                role,
                status,
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

    pub async fn update(&self, id: i64, name: &str, email: &str, password_hash: &str, role: &str, status: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user
            SET name = ?,
                email = ?,
                password_hash = ?,
                role = ?,
                status = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            name,
            email,
            password_hash,
            role,
            status,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }
    
    pub async fn update_status(&self, id: i64, status: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user
            SET status = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            status,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }
    
    pub async fn update_role(&self, id: i64, role: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user
            SET role = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            role,
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

        Ok(rows_affected > 0)
    }
    
    pub async fn find_all_by_status(&self, status: &str) -> Result<Vec<User>> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                name,
                email,
                password_hash,
                role,
                status,
                created_at,
                updated_at
            FROM user
            WHERE status = ?
            ORDER BY created_at DESC
            "#,
            status
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
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
                role,
                status,
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
