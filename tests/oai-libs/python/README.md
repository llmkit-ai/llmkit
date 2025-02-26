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
uv pip install openai
```

## Running the Examples

```bash
python python_examples.py
```

## Examples Included

- Basic chat completion
- Dynamic system prompt with JSON context
- One-shot prompt with dynamic system
- Dynamic system and user prompts
- JSON response format
- Streaming chat completion
- Multi-turn conversation

Each example demonstrates how to format inputs for the different prompt types in LLMKit, particularly showing how to use JSON-formatted inputs for template variable substitution in system and user prompts.