# Eval Runs - Rounds Feature Implementation

## Overview
Enhanced the evaluation system to support multiple "rounds" per eval run, allowing users to stress test prompts by running each evaluation multiple times with the same input to gather varied outputs.

## Core Pattern (Preserved)
The existing evaluation workflow remains unchanged:
1. Create new eval
2. **ONLY** when prompt version changes, user can re-run evaluations
3. Evaluate outcome/score results

## New Feature: Rounds Support
Users can now specify how many "rounds" to run each evaluation:
- Set rounds (e.g., 5) via UI input field
- Each round executes ALL evals for that prompt version
- Results are grouped by round with unique `run_id` per round
- Enables stress testing and variance analysis

## Files Modified

### Backend Changes

#### `/backend/src/controllers/prompt_eval_run.rs`
- **Lines 8-11**: Added `EvalRunParams` struct with optional `rounds` parameter
- **Lines 26-33**: Modified `execute_eval_run` to accept rounds query parameter (defaults to 1)
- **Lines 34-104**: Implemented loop to execute evaluations for specified number of rounds
- **Key Logic**: Each round gets unique `run_id` via `Uuid::new_v4()`, all evals run per round

### Frontend Changes

#### `/ui/types/response/prompt_eval_runs.ts`
- **Line 20**: Added optional `round_number?: number` field to `PromptEvalRunResponse` interface
- Maintains backward compatibility with existing eval runs

#### `/ui/composables/usePromptEvalRuns.ts`
- **Lines 21-58**: Updated `fetchEvalRunsByPromptVersion` to properly assign round numbers by grouping existing runs by `run_id` and sorting by creation time
- **Lines 67-84**: Updated `createEvalRun` function to accept rounds parameter and preserve round information from API response
- **API Call**: Updated to include `?rounds=${rounds}` query parameter
- **Round Assignment Logic**: Groups runs by `run_id`, sorts by creation time, assigns sequential round numbers

#### `/ui/components/evals/view-prompt-eval-input.vue`
- **Lines 54-56**: Added rounds input field with proper styling and labeling
- **Lines 72-78**: Added "Round" column header to eval results table
- **Lines 81-84**: Added round number display in table rows (shows `round_number || 1`)
- **Line 150**: Added `rounds` reactive variable (defaults to 1)
- **Line 198**: Updated `executeEvalRun()` to pass rounds parameter

## API Behavior

### Request
```
POST /v1/ui/prompt-eval-runs/{promptId}/version/{versionId}?rounds=5
```

### Response Structure
```json
[
  {
    "run_id": "uuid-round-1",
    "runs": [
      {
        "id": 1,
        "run_id": "uuid-round-1", 
        "prompt_eval_name": "eval1",
        "output": "result...",
        "score": null,
        ...
      }
    ]
  },
  {
    "run_id": "uuid-round-2", 
    "runs": [...]
  }
]
```

## UI Improvements

### Before
- Simple "New eval run" button
- Single execution per eval
- No round identification

### After  
- "Rounds: [input]" + "New eval run" button
- Configurable multiple executions (1-N rounds)
- Round column in results table showing which round each result belongs to
- Better visual organization of results

## Technical Details

### Round Processing
1. Backend creates unique `run_id` for each round
2. Frontend flattens grouped results while preserving round information
3. Each eval result tagged with `round_number` (1, 2, 3, etc.)
4. UI displays results in chronological order with round identification

### Data Flow
1. User sets rounds → Frontend sends `?rounds=N` parameter
2. Backend loops N times, each loop gets new `run_id`
3. Backend returns array of round groups
4. Frontend processes and flattens while adding `round_number`
5. UI displays with round column for easy identification

## Backward Compatibility
- Existing eval runs without rounds show as "Round 1"
- Default rounds value is 1 (maintains current behavior)
- All existing scoring and performance tracking works unchanged
- No database schema changes required (uses existing `run_id` field)