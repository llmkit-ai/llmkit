use serde::Serialize;

use crate::db::types::user::User;

#[derive(Debug, Clone, Serialize)]
pub struct MeResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for MeResponse {
    fn from(value: User) -> Self {
        MeResponse { 
            id: value.id, 
            name: value.name, 
            email: value.email, 
            created_at: value.created_at.to_string(), 
            updated_at: value.updated_at.to_string() 
        }
    }
}
