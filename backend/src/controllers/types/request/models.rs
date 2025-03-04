use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub name: String,
    pub provider_id: i64,
    pub supports_json: bool,
    pub supports_tools: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelRequest {
    pub name: String,
    pub provider_id: i64,
    pub supports_json: bool,
    pub supports_tools: bool,
}
