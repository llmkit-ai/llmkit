use sqlx::prelude::FromRow;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role_id: i64,
    pub registration_state_id: i64,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRoleRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct RegistrationStateRow {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserWithDetailsRow {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub role_id: i64,
    pub role_name: String, 
    pub registration_state_id: i64,
    pub registration_state: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Admin,
    Standard,
}

impl UserRole {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(UserRole::Admin),
            2 => Some(UserRole::Standard),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i64 {
        match self {
            UserRole::Admin => 1,
            UserRole::Standard => 2,
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "standard" => Some(UserRole::Standard),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::Standard => "standard".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationState {
    Pending,
    Approved,
    Rejected,
}

impl RegistrationState {
    pub fn from_id(id: i64) -> Option<Self> {
        match id {
            1 => Some(RegistrationState::Pending),
            2 => Some(RegistrationState::Approved),
            3 => Some(RegistrationState::Rejected),
            _ => None,
        }
    }

    pub fn to_id(&self) -> i64 {
        match self {
            RegistrationState::Pending => 1,
            RegistrationState::Approved => 2,
            RegistrationState::Rejected => 3,
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(RegistrationState::Pending),
            "approved" => Some(RegistrationState::Approved),
            "rejected" => Some(RegistrationState::Rejected),
            _ => None,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            RegistrationState::Pending => "pending".to_string(),
            RegistrationState::Approved => "approved".to_string(),
            RegistrationState::Rejected => "rejected".to_string(),
        }
    }
}
