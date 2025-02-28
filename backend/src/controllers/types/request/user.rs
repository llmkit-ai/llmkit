use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String
}
