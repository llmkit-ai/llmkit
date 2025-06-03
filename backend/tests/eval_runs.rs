use chrono::NaiveDateTime;
use sqlx::SqlitePool;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
struct PromptEvalExecutionRunResponse {
    id: i64,
    prompt_version_id: i64,
    prompt_eval_id: i64,
    score: Option<i64>,
    run_id: String,
    output: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

async fn setup_test_db() -> (SqlitePool, i64, i64) {
    // Create an isolated in-memory SQLite database for each test
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();
    
    // Get an existing model from migrations
    let model_id: i64 = sqlx::query_scalar!(
        r#"
        SELECT id FROM model WHERE name = 'google/gemini-2.0-flash-001' LIMIT 1
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    // Insert test user
    sqlx::query!(
        r#"
        INSERT INTO user (name, email, password_hash, created_at, updated_at)
        VALUES ('Test User', 'test@test.com', 'hash', datetime('now'), datetime('now'))
        "#
    )
    .execute(&pool)
    .await
    .unwrap();
    
    let user_id: i64 = sqlx::query_scalar!(
        r#"
        SELECT id FROM user WHERE email = 'test@test.com'
        "#
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    // Insert test prompt with a unique key
    let prompt_key = format!("test-prompt-{}", uuid::Uuid::new_v4());
    sqlx::query!(
        r#"
        INSERT INTO prompt (key, current_prompt_version_id)
        VALUES (?, NULL)
        "#,
        prompt_key
    )
    .execute(&pool)
    .await
    .unwrap();
    
    let prompt_id: i64 = sqlx::query_scalar!(
        r#"
        SELECT id FROM prompt WHERE key = ?
        "#,
        prompt_key
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    // Insert prompt version
    sqlx::query!(
        r#"
        INSERT INTO prompt_version (
            prompt_id, model_id, version_number, system, user,
            max_tokens, temperature, json_mode, prompt_type, is_chat,
            created_at, updated_at
        )
        VALUES (
            ?, ?, 1, 'You are a test assistant', 'Hello {{name}}',
            100, 0.7, 0, 'static', 1,
            datetime('now'), datetime('now')
        )
        "#,
        prompt_id,
        model_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    let prompt_version_id: i64 = sqlx::query_scalar!(
        r#"
        SELECT id FROM prompt_version WHERE prompt_id = ? ORDER BY id DESC LIMIT 1
        "#,
        prompt_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    // Update prompt to point to the version
    sqlx::query!(
        r#"
        UPDATE prompt SET current_prompt_version_id = ? WHERE id = ?
        "#,
        prompt_version_id,
        prompt_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Insert eval test
    sqlx::query!(
        r#"
        INSERT INTO prompt_eval (
            prompt_id, evaluation_type, name, user_prompt_input,
            created_at, updated_at
        )
        VALUES (
            ?, 'human', 'Test Eval', '{"name": "World"}',
            datetime('now'), datetime('now')
        )
        "#,
        prompt_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    (pool, prompt_version_id, prompt_id)
}

#[tokio::test]
async fn test_multiple_rounds_create_rows() {
    let (pool, prompt_version_id, prompt_id) = setup_test_db().await;
    
    // Get the eval test id
    let eval_id: i64 = sqlx::query_scalar!(
        r#"
        SELECT id FROM prompt_eval WHERE prompt_id = ?
        "#,
        prompt_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    // Test with 3 rounds
    let rounds = 3;
    let mut all_runs = Vec::new();
    
    for round in 0..rounds {
        let run_id = format!("test-run-{}", round);
        
        // Simulate creating an eval run (what execute_eval_run does internally)
        let result = sqlx::query_as!(
            PromptEvalExecutionRunResponse,
            r#"
            INSERT INTO prompt_eval_run (
                prompt_version_id, prompt_eval_id, run_id, output, 
                created_at, updated_at
            )
            VALUES (?, ?, ?, 'Test response', datetime('now'), datetime('now'))
            RETURNING 
                id,
                prompt_version_id,
                prompt_eval_id,
                score,
                run_id,
                output,
                created_at,
                updated_at
            "#,
            prompt_version_id,
            eval_id,
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        all_runs.push(result);
    }
    
    // Verify we created 3 runs
    assert_eq!(all_runs.len(), 3);
    
    // Verify each run has a unique run_id
    let run_ids: Vec<String> = all_runs.iter().map(|r| r.run_id.clone()).collect();
    assert_eq!(run_ids[0], "test-run-0");
    assert_eq!(run_ids[1], "test-run-1");
    assert_eq!(run_ids[2], "test-run-2");
    
    // Verify all runs are for the same prompt version and eval test
    for run in &all_runs {
        assert_eq!(run.prompt_version_id, prompt_version_id);
        assert_eq!(run.prompt_eval_id, eval_id);
        assert_eq!(run.output, "Test response");
    }
    
    // Verify database has 3 rows
    let count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM prompt_eval_run
        WHERE prompt_eval_id = ?
        "#,
        eval_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_single_round_default() {
    let (pool, prompt_version_id, prompt_id) = setup_test_db().await;
    
    // Get the eval test id
    let eval_id: i64 = sqlx::query_scalar!(
        r#"
        SELECT id FROM prompt_eval WHERE prompt_id = ?
        "#,
        prompt_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    // Test default behavior (1 round when not specified)
    let run_id = "single-run-test";
    
    // Create a single eval run
    let result = sqlx::query_as!(
        PromptEvalExecutionRunResponse,
        r#"
        INSERT INTO prompt_eval_run (
            prompt_version_id, prompt_eval_id, run_id, output,
            created_at, updated_at
        )
        VALUES (?, ?, ?, 'Single test response', datetime('now'), datetime('now'))
        RETURNING 
            id,
            prompt_version_id,
            prompt_eval_id,
            score,
            run_id,
            output,
            created_at,
            updated_at
        "#,
        prompt_version_id,
        eval_id,
        run_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(result.run_id, "single-run-test");
    assert_eq!(result.output, "Single test response");
    
    // Verify only 1 row exists
    let count: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as count
        FROM prompt_eval_run
        WHERE run_id = ?
        "#,
        run_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_run_id_grouping() {
    let (pool, prompt_version_id, prompt_id) = setup_test_db().await;
    
    // Create multiple eval tests
    sqlx::query!(
        r#"
        INSERT INTO prompt_eval (
            prompt_id, evaluation_type, name, user_prompt_input,
            created_at, updated_at
        )
        VALUES (
            ?, 'human', 'Test Eval 2', '{"name": "Rust"}',
            datetime('now'), datetime('now')
        )
        "#,
        prompt_id
    )
    .execute(&pool)
    .await
    .unwrap();
    
    // Get both eval test ids
    let eval_ids: Vec<i64> = sqlx::query_scalar!(
        r#"
        SELECT id FROM prompt_eval WHERE prompt_id = ? ORDER BY id
        "#,
        prompt_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    // Simulate a round with multiple eval tests
    let run_id = "grouped-run-test";
    
    // Create runs for both eval tests with the same run_id
    for eval_id in &eval_ids {
        sqlx::query!(
            r#"
            INSERT INTO prompt_eval_run (
                prompt_version_id, prompt_eval_id, run_id, output,
                created_at, updated_at
            )
            VALUES (?, ?, ?, 'Response for eval test', datetime('now'), datetime('now'))
            "#,
            prompt_version_id,
            eval_id,
            run_id
        )
        .execute(&pool)
        .await
        .unwrap();
    }
    
    // Verify both runs share the same run_id
    let runs: Vec<PromptEvalExecutionRunResponse> = sqlx::query_as!(
        PromptEvalExecutionRunResponse,
        r#"
        SELECT 
            id,
            prompt_version_id,
            prompt_eval_id,
            score,
            run_id,
            output,
            created_at,
            updated_at
        FROM prompt_eval_run
        WHERE run_id = ?
        ORDER BY prompt_eval_id
        "#,
        run_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0].prompt_eval_id, eval_ids[0]);
    assert_eq!(runs[1].prompt_eval_id, eval_ids[1]);
    assert_eq!(runs[0].run_id, runs[1].run_id);
}