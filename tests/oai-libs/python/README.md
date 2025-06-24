# Python OpenAI Client Examples for LLMKit

This directory contains examples showing how to use the OpenAI Python client library with LLMKit.

## Prerequisites

- Python 3.7+
- LLMKit server running locally on port 8000
- uv (for dependency management)

## Setup

Using uv:
```bash
uv venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
uv pip install openai pytest responses requests-mock
```

## Running the Examples

```bash
python main.py
```

## Running the Fallback Tests

Test the fallback functionality (OpenRouter → OpenAI):
```bash
# Run with pytest (recommended)
pytest simple_fallback_test.py -v -s

# Or run directly
python run_fallback_test.py

# Or run the test file directly
python simple_fallback_test.py
```

## Examples Included

### Basic Examples (`main.py`)
- Basic chat completion
- Dynamic system prompt with JSON context
- One-shot prompt with dynamic system
- Dynamic system and user prompts
- JSON response format
- Streaming chat completion
- Multi-turn conversation

### Fallback Tests (`simple_fallback_test.py`)
- **OpenRouter Rate Limit → OpenAI Fallback**: Tests automatic failover when primary provider hits rate limits
- **Disabled Fallback**: Verifies fallback doesn't occur when disabled
- **Fallback Exhausted**: Tests scenario where all providers fail
- **Configuration Validation**: Validates fallback configuration structure

Each example demonstrates how to format inputs for the different prompt types in LLMKit, particularly showing how to use JSON-formatted inputs for template variable substitution in system and user prompts.

## Fallback Configuration

The fallback tests demonstrate how to configure automatic provider switching:

```python
fallback_config = {
    "enabled": True,
    "providers": [
        {
            "provider": "openrouter",
            "model_name": "openai/gpt-3.5-turbo",
            "base_url": "https://openrouter.ai/api/v1",
            "catch_errors": ["rate_limit", "auth"]
        },
        {
            "provider": "openai", 
            "model_name": "gpt-3.5-turbo",
            "catch_errors": ["all"]
        }
    ],
    "max_retries_per_provider": 3
}
```

This configuration will:
1. Try OpenRouter first
2. If OpenRouter fails with rate limit or auth errors, fallback to OpenAI
3. Retry each provider up to 3 times before moving to the next