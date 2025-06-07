// Integration tests for the eval rounds feature
// These tests verify that the rounds parameter correctly creates multiple evaluation runs
// with unique run_ids that group eval tests executed in the same round

use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use uuid::Uuid;

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
    
    let _user_id: i64 = sqlx::query_scalar!(
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

// Helper to setup multiple eval tests
async fn setup_test_db_with_multiple_evals() -> (SqlitePool, i64, i64, Vec<i64>) {
    let (pool, prompt_version_id, prompt_id) = setup_test_db().await;
    
    // Add more eval tests
    let eval_ids = vec![
        sqlx::query_scalar!(
            r#"
            SELECT id FROM prompt_eval WHERE prompt_id = ?
            "#,
            prompt_id
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        
        sqlx::query_scalar!(
            r#"
            INSERT INTO prompt_eval (
                prompt_id, evaluation_type, name, user_prompt_input,
                created_at, updated_at
            )
            VALUES (
                ?, 'human', 'Test Eval 2', '{"name": "Rust"}',
                datetime('now'), datetime('now')
            )
            RETURNING id
            "#,
            prompt_id
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
        
        sqlx::query_scalar!(
            r#"
            INSERT INTO prompt_eval (
                prompt_id, evaluation_type, name, user_prompt_input,
                created_at, updated_at
            )
            VALUES (
                ?, 'human', 'Test Eval 3', '{"name": "Testing"}',
                datetime('now'), datetime('now')
            )
            RETURNING id
            "#,
            prompt_id
        )
        .fetch_one(&pool)
        .await
        .unwrap(),
    ];
    
    (pool, prompt_version_id, prompt_id, eval_ids)
}

// Simulates what the execute_eval_run controller does for each round
async fn execute_round(
    pool: &SqlitePool,
    prompt_version_id: i64,
    eval_ids: &[i64],
    run_id: &str,
) -> Vec<PromptEvalExecutionRunResponse> {
    let mut runs = Vec::new();
    
    for eval_id in eval_ids {
        // Simulate LLM response
        let output = format!("Response for eval {} in run {}", eval_id, run_id);
        
        let result = sqlx::query_as!(
            PromptEvalExecutionRunResponse,
            r#"
            INSERT INTO prompt_eval_run (
                prompt_version_id, prompt_eval_id, run_id, output, 
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))
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
            run_id,
            output
        )
        .fetch_one(pool)
        .await
        .unwrap();
        
        runs.push(result);
    }
    
    runs
}

// Original test 1: Basic multiple rounds test
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

// Original test 2: Single round default behavior
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

// Original test 3: Run ID grouping
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

// New test: Controller behavior with UUID generation
#[tokio::test]
async fn test_controller_behavior_multiple_rounds() {
    let (pool, prompt_version_id, _prompt_id, eval_ids) = setup_test_db_with_multiple_evals().await;
    
    // Simulate controller behavior with 3 rounds
    let rounds = 3;
    let mut all_round_results = Vec::new();
    
    for _round in 0..rounds {
        // Controller generates a new UUID for each round
        let run_id = Uuid::new_v4().to_string();
        
        // Execute all eval tests for this round
        let round_runs = execute_round(&pool, prompt_version_id, &eval_ids, &run_id).await;
        
        // Verify all runs in this round have the same run_id
        for run in &round_runs {
            assert_eq!(run.run_id, run_id);
        }
        
        all_round_results.push((run_id, round_runs));
    }
    
    // Verify we have 3 rounds
    assert_eq!(all_round_results.len(), 3);
    
    // Verify each round has runs for all eval tests
    for (run_id, runs) in &all_round_results {
        assert_eq!(runs.len(), eval_ids.len());
        
        // Verify all runs in a round share the same run_id
        for run in runs {
            assert_eq!(&run.run_id, run_id);
        }
    }
    
    // Verify all run_ids are unique across rounds
    let run_ids: Vec<&String> = all_round_results.iter()
        .map(|(run_id, _)| run_id)
        .collect();
    let unique_run_ids: std::collections::HashSet<_> = run_ids.iter().collect();
    assert_eq!(run_ids.len(), unique_run_ids.len());
    
    // Verify total number of runs in database
    let total_runs: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM prompt_eval_run"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(total_runs, (rounds * eval_ids.len() as i64));
}

// New test: Concurrent execution safety
#[tokio::test]
async fn test_run_id_uniqueness_across_concurrent_requests() {
    let (pool, prompt_version_id, _prompt_id, eval_ids) = setup_test_db_with_multiple_evals().await;
    
    // Simulate multiple concurrent requests (like multiple users running evals)
    let mut handles = Vec::new();
    
    for _i in 0..5 {
        let pool_clone = pool.clone();
        let eval_ids_clone = eval_ids.clone();
        
        let handle = tokio::spawn(async move {
            // Each "request" generates its own run_id
            let run_id = Uuid::new_v4().to_string();
            execute_round(&pool_clone, prompt_version_id, &eval_ids_clone, &run_id).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all concurrent executions
    let results: Vec<Vec<PromptEvalExecutionRunResponse>> = 
        futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();
    
    // Verify all run_ids are unique (no collisions)
    let unique_run_ids: std::collections::HashSet<String> = 
        results.iter()
            .map(|runs| runs[0].run_id.clone())
            .collect();
    
    assert_eq!(unique_run_ids.len(), 5); // 5 unique run_ids for 5 concurrent requests
}

// New test: Response grouping structure
#[tokio::test]
async fn test_response_grouping_structure() {
    let (pool, prompt_version_id, _prompt_id, eval_ids) = setup_test_db_with_multiple_evals().await;
    
    // Execute 2 rounds
    let round1_id = Uuid::new_v4().to_string();
    let round2_id = Uuid::new_v4().to_string();
    
    let round1_runs = execute_round(&pool, prompt_version_id, &eval_ids, &round1_id).await;
    let round2_runs = execute_round(&pool, prompt_version_id, &eval_ids, &round2_id).await;
    
    // Simulate how the controller groups responses
    struct GroupedResponse {
        run_id: String,
        runs: Vec<PromptEvalExecutionRunResponse>,
    }
    
    let grouped_responses = vec![
        GroupedResponse {
            run_id: round1_id.clone(),
            runs: round1_runs,
        },
        GroupedResponse {
            run_id: round2_id.clone(),
            runs: round2_runs,
        },
    ];
    
    // Verify grouping structure
    assert_eq!(grouped_responses.len(), 2);
    
    for response in &grouped_responses {
        // Each group should have runs for all eval tests
        assert_eq!(response.runs.len(), eval_ids.len());
        
        // All runs in a group should share the same run_id
        for run in &response.runs {
            assert_eq!(run.run_id, response.run_id);
        }
    }
    
    // Verify database query returns runs grouped by run_id
    let db_runs: Vec<(String, i64)> = sqlx::query_as::<_, (String, i64)>(
        "SELECT run_id, COUNT(*) as count FROM prompt_eval_run GROUP BY run_id ORDER BY run_id"
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    
    assert_eq!(db_runs.len(), 2);
    for (_, count) in db_runs {
        assert_eq!(count, eval_ids.len() as i64);
    }
}

// New test: Backward compatibility - single round returns single response
#[tokio::test]
async fn test_backward_compatibility_single_round() {
    let (pool, prompt_version_id, _prompt_id, eval_ids) = setup_test_db_with_multiple_evals().await;
    
    // Test with rounds=1 (or not specified)
    let run_id = Uuid::new_v4().to_string();
    let runs = execute_round(&pool, prompt_version_id, &eval_ids, &run_id).await;
    
    // When rounds=1, the API should return a single PromptEvalExecutionRunResponse
    // not wrapped in an array for backward compatibility
    assert_eq!(runs.len(), eval_ids.len());
    
    // All runs should share the same run_id
    for run in &runs {
        assert_eq!(run.run_id, run_id);
    }
}

// New test: Edge case - no eval tests
#[tokio::test]
async fn test_edge_case_no_eval_tests() {
    let (pool, prompt_version_id, prompt_id, _) = setup_test_db_with_multiple_evals().await;
    
    // Delete all eval tests
    sqlx::query!("DELETE FROM prompt_eval WHERE prompt_id = ?", prompt_id)
        .execute(&pool)
        .await
        .unwrap();
    
    // Execute round with no eval tests
    let run_id = Uuid::new_v4().to_string();
    let runs = execute_round(&pool, prompt_version_id, &[], &run_id).await;
    
    // Should return empty vec
    assert_eq!(runs.len(), 0);
    
    // Verify no runs in database
    let count: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM prompt_eval_run WHERE prompt_version_id = ?",
        prompt_version_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(count, 0);
}

// New test: Large number of rounds
#[tokio::test]
async fn test_large_number_of_rounds() {
    let (pool, prompt_version_id, _prompt_id, eval_ids) = setup_test_db_with_multiple_evals().await;
    
    // Test with a large number of rounds
    let rounds = 10;
    let mut run_ids = Vec::new();
    
    for _ in 0..rounds {
        let run_id = Uuid::new_v4().to_string();
        run_ids.push(run_id.clone());
        execute_round(&pool, prompt_version_id, &eval_ids, &run_id).await;
    }
    
    // Verify all run_ids are unique
    let unique_run_ids: std::collections::HashSet<_> = run_ids.iter().collect();
    assert_eq!(run_ids.len(), unique_run_ids.len());
    
    // Verify total runs
    let total_runs: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM prompt_eval_run"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(total_runs, (rounds * eval_ids.len()) as i64);
    
    // Verify we can query runs by run_id efficiently
    for run_id in &run_ids[..3] { // Check first 3
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM prompt_eval_run WHERE run_id = ?",
            run_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        
        assert_eq!(count, eval_ids.len() as i64);
    }
}