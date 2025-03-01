use axum::Json;
use jsonschema::meta;

use crate::controllers::types::{
    request::schema::ValidateSchemaRequest,
    response::schema::ValidateSchemaResponse,
};
use crate::AppError;

pub async fn validate_schema(
    Json(payload): Json<ValidateSchemaRequest>,
) -> Result<Json<ValidateSchemaResponse>, AppError> {
    let schema = &payload.schema;

    // Validate the schema against JSON Schema meta-schema
    match meta::validate(schema) {
        Ok(_) => {
            // Schema is valid
            let response = ValidateSchemaResponse {
                valid: true,
                errors: None,
            };
            Ok(Json(response))
        }
        Err(err) => {
            // Schema is invalid but this is not an error from the API perspective
            // We're just telling the client that their schema is invalid
            let errors = vec![err.to_string()];
            let response = ValidateSchemaResponse {
                valid: false,
                errors: Some(errors),
            };
            Ok(Json(response))
        }
    }
}
