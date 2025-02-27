use anyhow::Result;
use ring::pbkdf2;
use std::num::NonZeroU32;

use crate::db::types::user::{UserRow, UserWithDetailsRow, RegistrationState, UserRole};

static PBKDF2_ALG: pbkdf2::Algorithm = pbkdf2::PBKDF2_HMAC_SHA256;
const CREDENTIAL_LEN: usize = 32;
const SALT: &[u8] = b"llmkitusersalt"; // In a real app, use a unique salt per user
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

    pub fn hash_password(password: &str) -> String {
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

    pub fn verify_password(provided_password: &str, stored_hash: &str) -> bool {
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

    pub async fn create_user(
        &self, 
        email: &str, 
        name: &str, 
        password_hash: &str,
        role_id: Option<i64>,
        registration_state_id: Option<i64>
    ) -> Result<i64> {
        // Default to standard role (2) and pending state (1) if not provided
        let role = role_id.unwrap_or(2);
        let state = registration_state_id.unwrap_or(1);
        
        let id = sqlx::query!(
            r#"
            INSERT INTO user (
                email, 
                name, 
                password_hash, 
                role_id,
                registration_state_id,
                created_at, 
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id
            "#,
            email,
            name,
            password_hash,
            role,
            state,
        )
        .fetch_one(&self.pool)
        .await?
        .id;
        
        Ok(id)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<UserRow>> {
        let user = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, 
                email,
                name,
                password_hash,
                role_id,
                registration_state_id,
                created_at,
                updated_at
            FROM user
            WHERE email = ?
            LIMIT 1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn get_user_with_details_by_email(&self, email: &str) -> Result<Option<UserWithDetailsRow>> {
        let user = sqlx::query_as!(
            UserWithDetailsRow,
            r#"
            SELECT 
                u.id, 
                u.email,
                u.name,
                u.password_hash,
                u.role_id,
                r.name as role_name,
                u.registration_state_id,
                s.name as registration_state,
                u.created_at,
                u.updated_at
            FROM user u
            JOIN user_role r ON u.role_id = r.id
            JOIN registration_state s ON u.registration_state_id = s.id
            WHERE u.email = ?
            LIMIT 1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn get_user_by_id(&self, id: i64) -> Result<Option<UserRow>> {
        let user = sqlx::query_as!(
            UserRow,
            r#"
            SELECT 
                id, 
                email,
                name,
                password_hash,
                role_id,
                registration_state_id,
                created_at,
                updated_at
            FROM user
            WHERE id = ?
            LIMIT 1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn get_user_with_details_by_id(&self, id: i64) -> Result<Option<UserWithDetailsRow>> {
        let user = sqlx::query_as!(
            UserWithDetailsRow,
            r#"
            SELECT 
                u.id, 
                u.email,
                u.name,
                u.password_hash,
                u.role_id,
                r.name as role_name,
                u.registration_state_id,
                s.name as registration_state,
                u.created_at,
                u.updated_at
            FROM user u
            JOIN user_role r ON u.role_id = r.id
            JOIN registration_state s ON u.registration_state_id = s.id
            WHERE u.id = ?
            LIMIT 1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(user)
    }

    pub async fn list_users(&self) -> Result<Vec<UserWithDetailsRow>> {
        let users = sqlx::query_as!(
            UserWithDetailsRow,
            r#"
            SELECT 
                u.id, 
                u.email,
                u.name,
                u.password_hash,
                u.role_id,
                r.name as role_name,
                u.registration_state_id,
                s.name as registration_state,
                u.created_at,
                u.updated_at
            FROM user u
            JOIN user_role r ON u.role_id = r.id
            JOIN registration_state s ON u.registration_state_id = s.id
            ORDER BY u.id
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users)
    }

    pub async fn list_pending_users(&self) -> Result<Vec<UserWithDetailsRow>> {
        let pending_state_id = RegistrationState::Pending.to_id();
        
        let users = sqlx::query_as!(
            UserWithDetailsRow,
            r#"
            SELECT 
                u.id, 
                u.email,
                u.name,
                u.password_hash,
                u.role_id,
                r.name as role_name,
                u.registration_state_id,
                s.name as registration_state,
                u.created_at,
                u.updated_at
            FROM user u
            JOIN user_role r ON u.role_id = r.id
            JOIN registration_state s ON u.registration_state_id = s.id
            WHERE u.registration_state_id = ?
            ORDER BY u.created_at DESC
            "#,
            pending_state_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(users)
    }

    pub async fn update_user(&self, id: i64, name: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user 
            SET name = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            name,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }

    pub async fn update_password(&self, id: i64, password_hash: &str) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user 
            SET password_hash = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            password_hash,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }

    pub async fn update_user_role(&self, id: i64, role_id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user 
            SET role_id = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            role_id,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }

    pub async fn update_registration_state(&self, id: i64, state_id: i64) -> Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
            UPDATE user 
            SET registration_state_id = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            state_id,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        
        Ok(rows_affected > 0)
    }

    pub async fn approve_user(&self, id: i64) -> Result<bool> {
        let approved_state_id = RegistrationState::Approved.to_id();
        self.update_registration_state(id, approved_state_id).await
    }

    pub async fn reject_user(&self, id: i64) -> Result<bool> {
        let rejected_state_id = RegistrationState::Rejected.to_id();
        self.update_registration_state(id, rejected_state_id).await
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

    pub async fn is_admin(&self, id: i64) -> Result<bool> {
        let user = self.get_user_by_id(id).await?;
        
        if let Some(user) = user {
            Ok(user.role_id == UserRole::Admin.to_id())
        } else {
            Ok(false)
        }
    }

    pub async fn is_approved(&self, id: i64) -> Result<bool> {
        let user = self.get_user_by_id(id).await?;
        
        if let Some(user) = user {
            Ok(user.registration_state_id == RegistrationState::Approved.to_id())
        } else {
            Ok(false)
        }
    }
}
