# Evaluation Rounds Feature Summary

## Overview
This feature adds the ability to run prompt evaluations multiple times (rounds) to stress test prompts and gather more reliable performance metrics. Previously, evaluations could only be run once per prompt version change. With this feature, users can specify how many rounds to run, with each round executing all eval tests and grouping them with a unique `run_id`.

## Problem Statement
- **Current limitation**: Evaluations can only be re-run when the prompt version changes
- **Need**: Ability to run evaluations multiple times to:
  - Stress test prompts for consistency
  - Gather statistical data on prompt performance
  - Identify edge cases and variability in LLM responses

## Solution
Added an optional `rounds` parameter to the evaluation execution endpoint that allows running multiple rounds of evaluations, with each round identified by a unique UUID.

## Technical Implementation

### Backend Changes

#### 1. **Controller Updates** (`backend/src/controllers/prompt_eval_run.rs`)
- Added `EvalRunParams` struct with optional `rounds` parameter
- Modified `execute_eval_run` to:
  - Accept query parameter `?rounds=N` (defaults to 1)
  - Loop N times, generating a unique `run_id` (UUID) for each round
  - Execute all eval tests for each round
  - Return results maintaining backward compatibility:
    - Single `PromptEvalExecutionRunResponse` when rounds=1
    - Array of `PromptEvalExecutionRunResponse` when rounds>1

#### 2. **Response Types** (`backend/src/controllers/types/response/prompt_eval_run.rs`)
- Made `PromptEvalExecutionRunResponse` fields public for proper serialization
- Structure groups eval runs by their `run_id`

#### 3. **Tests** (`backend/tests/eval_runs.rs`)
- Added comprehensive test suite (724 lines) covering:
  - Multiple rounds execution
  - Run ID grouping behavior
  - Concurrent execution safety
  - Edge cases (no eval tests, large number of rounds)
  - Backward compatibility

### Frontend Changes

#### 1. **UI Component** (`ui/components/evals/view-prompt-eval-input.vue`)
- Added input field for specifying number of rounds
- Defaults to 1 round for backward compatibility

#### 2. **API Integration** (`ui/composables/usePromptEvalRuns.ts`)
- Updated `createEvalRun` to:
  - Pass rounds parameter to API
  - Handle both single response and array response types
  - Assign round numbers to runs based on creation order
- Modified `fetchEvalRunsByPromptVersion` to group runs by `run_id` and assign round numbers

#### 3. **Types** (`ui/types/response/prompt_eval_runs.ts`)
- Added optional `round_number` field to `PromptEvalRunResponse` for UI display

## API Changes

### Endpoint: `POST /v1/ui/prompt-eval-runs/{prompt_id}/version/{prompt_version_id}`

**Query Parameters:**
- `rounds` (optional, integer): Number of rounds to execute (default: 1)

**Response:**
- When `rounds=1`: Single `PromptEvalExecutionRunResponse` object (backward compatible)
- When `rounds>1`: Array of `PromptEvalExecutionRunResponse` objects

**Response Structure:**
```typescript
interface PromptEvalExecutionRunResponse {
  run_id: string;  // UUID identifying this round
  runs: PromptEvalRunResponse[];  // All eval test results for this round
}
```

## Benefits
1. **Statistical Analysis**: Run evaluations multiple times to gather performance statistics
2. **Stress Testing**: Identify edge cases and inconsistencies in prompt behavior
3. **Flexibility**: Users can choose how many rounds based on their testing needs
4. **Backward Compatible**: Existing integrations continue to work unchanged
5. **Clear Grouping**: Each round has a unique identifier for easy tracking

## Example Usage
```bash
# Run evaluation once (default behavior)
POST /v1/ui/prompt-eval-runs/123/version/456

# Run evaluation 5 times
POST /v1/ui/prompt-eval-runs/123/version/456?rounds=5
```

## Database Impact
No schema changes required. The feature uses the existing `run_id` column in `prompt_eval_run` table to group evaluations by round.

## Summary Statistics
- **Files Changed**: 6
- **Lines Added**: 864
- **Lines Modified**: 71
- **Test Coverage**: 9 comprehensive integration tests
- **Backward Compatibility**: ✅ Maintained