use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, delete},
    Router,
};
use axum::middleware as axum_middleware;

use middleware::auth;
use tracing_subscriber;

use anyhow::Result;
use controllers::{
    api_keys::{create_api_key, delete_api_key, list_api_keys},
    logs::{get_log, get_logs_count, list_logs},
    models::list_models,
    prompt_eval::{
        create_eval_test, delete_eval_test, get_eval_test_by_id, get_eval_test_by_prompt,
        update_eval_test,
    },
    prompt_eval_run::{
        execute_eval_run, get_eval_performance_by_prompt_id, get_eval_run_by_id,
        get_eval_runs_by_prompt_version, update_eval_run_score,
    },
    prompts::{
        create_prompt, delete_prompt, get_prompt, list_prompts, update_prompt,
        api_completions, api_completions_stream,
    },
};

use db::{init::DbData, types::prompt::PromptRowWithModel};
use moka::future::Cache;

pub mod common;
pub mod controllers;
pub mod db;
pub mod services;
pub mod middleware;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());

    tracing_subscriber::fmt().with_env_filter(log_level).init();

    let data = DbData::new(&database_url).await?;
    let app_state = AppState::new(data).await;

    // Build the router with all routes directly in main
    // Build the router with all routes directly in main
    let router = Router::new()
        .nest("/v1", Router::new()
            // API routes (OpenAI compatible) with API key auth
            .route("/chat/completions", post(api_completions))
                .route_layer(axum_middleware::from_fn_with_state(app_state.clone(), auth::api_key_middleware))
            .route("/chat/completions/stream", post(api_completions_stream))
                .route_layer(axum_middleware::from_fn_with_state(app_state.clone(), auth::api_key_middleware))

            // version
            .route("/", get(api_version_handler))
            
            // UI specific routes
            // Prompt routes
            .route("/ui/prompts", post(create_prompt).get(list_prompts))
            .route(
                "/ui/prompts/{id}",
                get(get_prompt).put(update_prompt).delete(delete_prompt),
            )
            .route("/ui/prompts/{id}/prompt-evals", get(get_eval_test_by_prompt))
            .route("/ui/prompts/{id}/performance", get(get_eval_performance_by_prompt_id))
            
            // Execute routes
            .route("/ui/prompts/execute", post(api_completions))
            .route("/ui/prompts/execute/stream", post(api_completions_stream))
            .route("/ui/prompts/execute/chat", post(api_completions))
            .route("/ui/prompts/execute/chat/stream", post(api_completions_stream))
            
            // Prompt evals routes
            .route("/ui/prompt-evals", post(create_eval_test))
            .route(
                "/ui/prompt-evals/{id}",
                get(get_eval_test_by_id)
                    .put(update_eval_test)
                    .delete(delete_eval_test),
            )
            
            // Prompt eval runs routes
            .route(
                "/ui/prompt-eval-runs/{prompt_id}/version/{prompt_version_id}",
                post(execute_eval_run).get(get_eval_runs_by_prompt_version),
            )
            .route("/ui/prompt-eval-runs/{id}", get(get_eval_run_by_id).put(update_eval_run_score))
            
            // Model routes
            .route("/ui/models", get(list_models))
            
            // Logs routes
            .route("/ui/logs", get(list_logs))
            .route("/ui/logs/{trace_id}", get(get_log))
            .route("/ui/logs/count", get(get_logs_count))
            
            // Settings routes
            .route("/ui/settings/api-keys", get(list_api_keys).post(create_api_key))
            .route("/ui/settings/api-keys/{id}", delete(delete_api_key))
            
            // User routes
            .route("/ui/auth/login", post(login))
            .route("/ui/users", get(get_users).post(create_user))
            .route("/ui/users/{id}", get(get_user).put(update_user).delete(delete_user))
        )
        .with_state(app_state);


    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn api_version_handler() -> &'static str {
    "llmkit api v0.1"
}

// APP STATE
#[derive(Clone)]
pub struct AppState {
    pub db: DbData,
    pub prompt_cache: Cache<i64, PromptRowWithModel>,
}

impl AppState {
    pub async fn new(data: DbData) -> Self {
        let prompt_cache: Cache<i64, PromptRowWithModel> = Cache::new(500);

        AppState {
            db: data,
            prompt_cache,
        }
    }
}

// ANYHOW ERROR HANDLING
#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    InternalServerError(String),
    TooManyRequests(String),
    Other(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized(e) => {
                tracing::error!("Unauthorized | error: {}", e);
                return (StatusCode::UNAUTHORIZED, format!("Unauthorized: {:?}", e))
                    .into_response();
            }
            AppError::BadRequest(e) => {
                tracing::error!("Bad Request | error: {}", e);
                return (StatusCode::BAD_REQUEST, format!("Bad Request: {:?}", e)).into_response();
            }
            AppError::NotFound(e) => {
                tracing::error!("Not Found | error: {}", e);
                return (StatusCode::NOT_FOUND, format!("{}", e)).into_response();
            }
            AppError::Conflict(e) => {
                tracing::error!("Conflict | error: {}", e);
                return (StatusCode::CONFLICT, format!("{}", e)).into_response();
            }
            AppError::InternalServerError(e) => {
                tracing::error!("Internal server error | error: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response();
            }
            AppError::TooManyRequests(e) => {
                tracing::error!("Too many requests | error: {}", e);
                return (StatusCode::TOO_MANY_REQUESTS, format!("{}", e)).into_response();
            }
            AppError::Other(e) => {
                tracing::error!("Internal Server Error | error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error"),
                )
                    .into_response();
            }
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Other(err)
    }
}