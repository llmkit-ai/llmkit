use axum::{
    body::Body,
    extract::{FromRequest, Request},
    http::StatusCode,
    Json,
};
use serde::de::DeserializeOwned;

pub struct JsonWithRequest<T>(pub T, pub Request);

impl<T, S> FromRequest<S> for JsonWithRequest<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
    Json<T>: FromRequest<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // Clone the request so we can extract the JSON and keep the request
        let (parts, body) = req.into_parts();
        let req_clone = Request::from_parts(parts.clone(), Body::empty());
        
        // Rebuild the original request
        let req = Request::from_parts(parts, body);
        
        // Extract the JSON
        let Json(payload) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_e| (StatusCode::BAD_REQUEST, "Invalid request JSON.".to_string()))?;
            
        Ok(JsonWithRequest(payload, req_clone))
    }
}
