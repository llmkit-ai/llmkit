# Phased Merge Plan for PR #30: Eval Rounds Feature

## Overview
This document outlines the plan to resolve merge conflicts for PR #30, which attempts to merge the `alchemiststudiosDOTai:master` branch (containing the eval rounds feature) into the upstream `llmkit-ai:master` branch.

## Current State Analysis

### Upstream Changes (llmkit-ai:master)
The upstream has added several major features since the fork:
1. **Azure Support** (#36) - New provider for Azure OpenAI
2. **Direct OpenAI Support** (#34) - Direct OpenAI integration
3. **Auto-expanding textarea fixes** (#33, #25)
4. **Streaming and Tool Calling fixes** (#29, #28, #26)

### Feature Branch Changes (alchemiststudiosDOTai:master)
The feature branch implements:
1. **Eval Rounds Support** - Ability to run evaluations multiple times
2. **Integration tests** for the rounds feature (backend/tests/eval_runs.rs)
3. **API parameter support** for specifying rounds count
4. **Run ID tracking** - Removed from general logging, kept only for eval runs
5. **Documentation cleanup** - Consolidated eval rounds info into README

## Conflict Analysis

### Primary Conflict File
- **File**: `backend/src/controllers/prompt_eval_run.rs`
- **Location**: Lines 47-75
- **Conflict Type**: Overlapping changes in the `execute_eval_run` function
- **Details**: 
  - Feature branch: Simplified user_content assignment (line 49)
  - Upstream: Restructured the code block and added max_tokens and temperature parameters to ChatCompletionRequest
  - Both versions create the same ChatCompletionRequest but with different formatting and parameter inclusion

### Related Changes (No Direct Conflicts)
These files have diverged but don't have merge conflicts:
1. **Provider System**:
   - New providers added upstream: `azure.rs`, `openai.rs`
   - Modified provider infrastructure in `providers/mod.rs`
   - Updated error handling in `llm.rs`
   
2. **Database Migrations**:
   - Upstream added: 
     - `20250603220312_add_openai_provider.sql`
     - `20250604220312_model_provider.sql`
     - `20250605000000_add_reasoning_effort.sql`
     - `20250606000000_azure_provider_support.sql`
   - Feature branch removed: 
     - `20250501000000_add_run_id_to_log.sql` (moved run_id to eval runs only)

3. **Frontend Components**:
   - UI updates for provider selection
   - Model management interfaces
   - Enhanced eval run UI with rounds support

4. **Documentation Changes**:
   - README.md updated with provider configuration details
   - Removed separate eval documentation (consolidated into README)

## Phased Merge Strategy

### Phase 1: Prepare Clean Base
1. Create a new branch from current docker branch
2. Document all custom changes in eval rounds feature
3. Backup current implementation

### Phase 2: Resolve Core Conflict
**File**: `backend/src/controllers/prompt_eval_run.rs`
- **Action**: Merge both changes:
  - Keep the rounds functionality from feature branch
  - Add max_tokens and temperature from upstream
  - Ensure proper error handling format from upstream

**Specific Changes**:
```rust
// Keep the feature branch's simplified approach but add upstream parameters
// Around line 49: Keep the simplified user_content assignment
let user_content = e.user_prompt_input.clone();

// Around lines 96-97: Add the upstream parameters
max_tokens: Some(prompt.max_tokens as u32),  // Add from upstream
temperature: Some(prompt.temperature as f32), // Add from upstream
```

### Phase 3: Provider Integration
1. **Accept all upstream provider changes**:
   - `backend/src/services/providers/azure.rs`
   - `backend/src/services/providers/openai.rs`
   - Provider registration in `mod.rs`

2. **Update provider infrastructure**:
   - Ensure eval rounds work with new providers
   - Test with Azure and OpenAI providers

### Phase 4: Database Migration Reconciliation
1. **Accept upstream migrations**:
   - All provider-related migrations from upstream
   - Note: Feature branch already removed the run_id migration (it's no longer needed)

2. **No migration conflicts expected**:
   - Feature branch cleaned up migrations in earlier commits
   - All upstream migrations should apply cleanly

### Phase 5: Frontend Updates
1. **Merge UI changes carefully**:
   - Keep eval rounds UI additions
   - Integrate new provider selection UI
   - Update model management components

## Quirks and Special Considerations

### 1. Error Message Format
- **Issue**: Error handling format differs slightly
- **Feature branch**: Already updated to use detailed error formatting
- **Upstream**: Similar detailed error formatting
- **Resolution**: Both versions now use similar error handling, minimal changes needed

### 2. Provider System Architecture
- **Issue**: Significant refactoring in upstream
- **Impact**: May affect how eval runs interact with providers
- **Resolution**: Test eval rounds with each new provider

### 3. Removed Features
- **Issue**: Feature branch removed some files that may exist upstream
- **Removed files**:
  - `backend/openapi.yaml` (unused documentation)
  - `backend/.dockerignore`
  - `backend/Dockerfile.fast`
  - `ui/pages/docs/eval.md` (consolidated into README)
- **Resolution**: Keep these files removed as per feature branch cleanup

### 4. Package Lock Files
- **Issue**: Both `package-lock.json` and `bun.lock` modified
- **Resolution**: Regenerate after merge

### 5. Docker Configuration
- **Issue**: Feature branch has different Docker configuration
- **Changes**: Simplified Dockerfile, removed multi-stage builds
- **Resolution**: Review which Docker configuration is more appropriate

## Testing Plan Post-Merge

1. **Unit Tests**:
   - Run existing eval rounds tests
   - Add tests for new providers + rounds

2. **Integration Tests**:
   - Test rounds with each provider (OpenRouter, OpenAI, Azure)
   - Verify migration sequence

3. **Manual Testing**:
   - Create eval with multiple rounds
   - Test with different providers
   - Verify UI functionality

## Risk Assessment

**High Risk**:
- Provider integration with eval rounds (new providers need testing)
- Merge conflict resolution in `prompt_eval_run.rs`

**Medium Risk**:
- UI component integration
- Docker configuration differences

**Low Risk**:
- Documentation updates (already consolidated)
- Package dependency updates
- Error handling (already aligned)

## Recommended Merge Order

1. First merge: Core conflict in `prompt_eval_run.rs`
2. Accept all provider additions
3. Reconcile migrations
4. Update frontend components
5. Fix package dependencies
6. Run full test suite

## Notes
- The eval rounds feature is relatively isolated, making integration easier
- Most upstream changes are additive (new providers)
- Main complexity is ensuring rounds work with all providers