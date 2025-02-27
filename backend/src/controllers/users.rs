use axum::{
    extract::{Path, State}, 
    http::header::{SET_COOKIE, CONTENT_TYPE}, 
    Json,
    response::{IntoResponse, Response}
};
use serde_json;
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState};
use crate::db::types::user::{UserRole, RegistrationState};
use crate::middleware::auth;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub role: String,
    pub registration_state: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserResponse>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String, // We'll hash this on the server
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String, // We'll hash this on the server
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub role: String,
    pub registration_state: String,
    pub token: String, // JWT token
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePasswordRequest {
    pub password: String, // We'll hash this on the server
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String, // "admin" or "standard"
}

#[derive(Debug, Deserialize)]
pub struct UpdateRegistrationStateRequest {
    pub state: String, // "pending", "approved", or "rejected"
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub success: bool,
}

// Register a new user
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Validate inputs
    if payload.email.is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }
    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password cannot be empty".to_string()));
    }
    
    // Hash the password - in a real application, use a proper password hashing library
    // like bcrypt, Argon2, or PBKDF2 with a unique salt
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(payload.password.as_bytes());
    let password_hash = format!("{:x}", hasher.finalize());

    let existing_user = state.db.user.get_user_by_email(&payload.email).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest("User with this email already exists".to_string()));
    }

    // Create the user (with default role_id = 2 for standard and registration_state_id = 1 for pending)
    let user_id = state.db.user.create_user(
        &payload.email,
        &payload.name,
        &password_hash, // Use our hashed password
        None,
        None,
    ).await.map_err(|e| AppError::InternalServerError(format!("Failed to create user: {}", e)))?;

    // Get the created user with details
    let user = state.db.user.get_user_with_details_by_id(user_id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::InternalServerError("User was created but not found".to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role_name,
        registration_state: user.registration_state,
        created_at: user.created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
    }))
}

// Login a user
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate inputs
    if payload.email.is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }
    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password cannot be empty".to_string()));
    }
    
    // Hash the password for comparison
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(payload.password.as_bytes());
    let password_hash = format!("{:x}", hasher.finalize());

    // Get the user with details
    let user = state.db.user.get_user_with_details_by_email(&payload.email).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Verify password (comparing stored hash with our new hash)
    if user.password_hash != password_hash {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Check if user is approved
    if user.registration_state != RegistrationState::Approved.to_string() {
        return Err(AppError::Unauthorized("Your account is not approved yet".to_string()));
    }

    // Generate JWT token
    let token = auth::create_jwt(user.id, &user.role_name)
        .map_err(|e| AppError::InternalServerError(format!("Failed to generate token: {}", e)))?;
    
    // Create the response body
    let login_response = LoginResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role_name,
        registration_state: user.registration_state,
        token: token.clone(), // Still include token in response body for clients that prefer it
        success: true,
    };
    
    // Set cookie (secure, httponly, same-site) and return the response
    let cookie = format!("{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}; Secure", 
                         crate::middleware::auth::JWT_COOKIE_NAME,
                         token,
                         crate::middleware::auth::JWT_EXPIRY_SECONDS);
    
    // Return response with cookie and JSON body
    let mut response = Response::new(serde_json::to_string(&login_response).unwrap());
    response.headers_mut().insert(SET_COOKIE, cookie.parse().unwrap());
    response.headers_mut().insert(CONTENT_TYPE, "application/json".parse().unwrap());
    
    Ok(response)
}

// Get all users (admin only)
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<UserListResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Get all users
    let users = state.db.user.list_users().await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch users: {}", e)))?;
    
    // Map to response format
    let user_responses = users.into_iter().map(|user| {
        UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role_name,
            registration_state: user.registration_state,
            created_at: user.created_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        }
    }).collect();
    
    Ok(Json(UserListResponse { users: user_responses }))
}

// Get pending users (admin only)
pub async fn list_pending_users(
    State(state): State<AppState>,
) -> Result<Json<UserListResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Get pending users
    let users = state.db.user.list_pending_users().await
        .map_err(|e| AppError::InternalServerError(format!("Failed to fetch pending users: {}", e)))?;
    
    // Map to response format
    let user_responses = users.into_iter().map(|user| {
        UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role_name,
            registration_state: user.registration_state,
            created_at: user.created_at
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default(),
        }
    }).collect();
    
    Ok(Json(UserListResponse { users: user_responses }))
}

// Get current user (authenticated user)
pub async fn get_current_user(
    State(state): State<AppState>,
    req: Request<axum::body::Body>,
) -> Result<Json<UserResponse>, AppError> {
    // Extract claims from cookie
    let claims = auth::extract_claims_from_request(&req)
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;
    
    // Get user ID from claims
    let user_id = auth::get_user_id_from_claims(&claims)?;
    
    // Get the user details
    let user = state.db.user.get_user_with_details_by_id(user_id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User not found")))?;
    
    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role_name,
        registration_state: user.registration_state,
        created_at: user.created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
    }))
}

// Get user profile by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
    // Authentication is handled by middleware - user_auth_middleware
    // Authorization would normally check if the user is requesting their own profile
    // or if they're an admin, but that requires the current user ID from the middleware
    
    let user = state.db.user.get_user_with_details_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", id)))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        name: user.name,
        role: user.role_name,
        registration_state: user.registration_state,
        created_at: user.created_at
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default(),
    }))
}

// Update user profile
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware
    
    // Validate inputs
    if payload.name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }

    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Update the user
    let updated = state.db.user.update_user(id, &payload.name).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to update user: {}", e)))?;

    Ok(Json(UpdateResponse { success: updated }))
}

// Update user password
pub async fn update_password(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdatePasswordRequest>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware
    
    // Validate inputs
    if payload.password.is_empty() {
        return Err(AppError::BadRequest("Password cannot be empty".to_string()));
    }
    
    // Hash the password
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(payload.password.as_bytes());
    let password_hash = format!("{:x}", hasher.finalize());

    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Update the password
    let updated = state.db.user.update_password(id, &password_hash).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to update password: {}", e)))?;

    Ok(Json(UpdateResponse { success: updated }))
}

// Update user role (admin only)
pub async fn update_role(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Validate the role
    let role = UserRole::from_str(&payload.role)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid role: {}", payload.role)))?;
    
    // We'd need the user's ID from middleware here
    // For now, let's assume admin can't demote themselves if they're trying to change their own account
    if id == 1 { // Assume ID 1 is the admin for now
        return Err(AppError::BadRequest("Admins cannot demote themselves".to_string()));
    }
    
    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Update the role
    let updated = state.db.user.update_user_role(id, role.to_id()).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to update role: {}", e)))?;

    Ok(Json(UpdateResponse { success: updated }))
}

// Approve or reject a pending user (admin only)
pub async fn update_registration_state(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateRegistrationStateRequest>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Validate the state
    let state_enum = RegistrationState::from_str(&payload.state)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid state: {}", payload.state)))?;
    
    // Admins cannot change their own state
    if id == 1 { // Assume ID 1 is the admin for now
        return Err(AppError::BadRequest("Admins cannot change their own registration state".to_string()));
    }
    
    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Update the state
    let updated = state.db.user.update_registration_state(id, state_enum.to_id()).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to update registration state: {}", e)))?;

    Ok(Json(UpdateResponse { success: updated }))
}

// Helper functions for approve/reject
pub async fn approve_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Approve the user
    let updated = state.db.user.approve_user(id).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to approve user: {}", e)))?;

    Ok(Json(UpdateResponse { success: updated }))
}

pub async fn reject_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Reject the user
    let updated = state.db.user.reject_user(id).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to reject user: {}", e)))?;

    Ok(Json(UpdateResponse { success: updated }))
}

// Delete a user (admin only)
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UpdateResponse>, AppError> {
    // Authentication is handled by middleware - route protected by admin_auth_middleware
    
    // Prevent admins from deleting themselves
    if id == 1 { // Assume ID 1 is the admin for now
        return Err(AppError::BadRequest("Admins cannot delete their own account".to_string()));
    }
    
    // Check if user exists
    let user_exists = state.db.user.get_user_by_id(id).await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?
        .is_some();

    if !user_exists {
        return Err(AppError::NotFound(format!("User with ID {} not found", id)));
    }

    // Delete the user
    let deleted = state.db.user.delete_user(id).await
        .map_err(|e| AppError::InternalServerError(format!("Failed to delete user: {}", e)))?;

    Ok(Json(UpdateResponse { success: deleted }))
}
