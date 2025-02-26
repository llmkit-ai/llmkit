# Node.js OpenAI Client Examples for LLMKit

This directory contains examples showing how to use the OpenAI Node.js client library with LLMKit.

## Prerequisites

- Node.js 16+
- npm or bun
- LLMKit server running locally on port 8000

## Setup

```bash
# Using npm
npm install

# Using bun
bun install
```

## Running the Examples

```bash
# Using npm
npm start

# Using bun
bun start
```

## Examples Included

- Basic chat completion
- Dynamic system prompt with JSON context
- One-shot dynamic system prompt
- Dynamic system and user prompts
- JSON response format
- Streaming chat completion
- Function calling
- Multi-turn conversation with context

Each example demonstrates how to format inputs for the different prompt types in LLMKit, with special attention to JSON template variable substitution for both system and user prompts.