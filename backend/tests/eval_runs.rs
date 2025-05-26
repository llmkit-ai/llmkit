use backend::{controllers::prompt_eval_run::{execute_eval_run, EvalRunParams}, AppState, db::init::DbData};
use axum::extract::{Path, Query, State};

#[tokio::test]
async fn multiple_rounds_create_rows() {
    let data = DbData::new("sqlite::memory:").await.unwrap();
    let state = AppState::new(data.clone()).await;

    // use first prompt from seeded data
    let prompt = data.prompt.list_prompts().await.unwrap().first().cloned().unwrap();

    // create eval for prompt
    let _ = data.prompt_eval.create(prompt.id, None, "hi".to_string(), "human", Some("t".to_string())).await.unwrap();

    let params = EvalRunParams { rounds: Some(2) };
    let _ = execute_eval_run(
        Path((prompt.id, prompt.version_id)),
        State(state.clone()),
        Query(params),
    ).await.unwrap();

    let runs = data.prompt_eval_run.get_by_prompt_version(prompt.version_id).await.unwrap();
    assert_eq!(runs.len(), 2);
}
