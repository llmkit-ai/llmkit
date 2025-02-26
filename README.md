# LLMKit

LLMKit is a comprehensive toolkit for managing, testing, and deploying LLM prompts with a focus on versioning, evaluation, and developer-friendly workflows.

![LLMKit Banner](docs/images/banner.png) <!-- [SCREENSHOT PLACEHOLDER] -->

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [How It Works](#how-it-works)
  - [Prompt Architecture](#prompt-architecture)
  - [Template Variables](#template-variables)
  - [OpenAI Compatibility](#openai-compatibility)
  - [Prompt Evaluation](#prompt-evaluation)
- [Technical Stack](#technical-stack)
- [Setup and Installation](#setup-and-installation)
  - [Backend Setup](#backend-setup)
  - [Frontend Setup](#frontend-setup)
- [Usage](#usage)
  - [Managing Prompts](#managing-prompts)
  - [Testing Prompts](#testing-prompts)
  - [API Integration](#api-integration)
- [Contributing](#contributing)
- [License](#license)

## Overview

LLMKit provides a unified interface for working with various LLM providers (OpenAI, Anthropic, Google Gemini, etc.) through a consistent API. It solves several key challenges in LLM prompt engineering:

- **Prompt Versioning**: Track changes to prompts over time
- **Template Management**: Use powerful templating for dynamic prompts
- **Performance Evaluation**: Test and measure prompt effectiveness
- **Provider Abstraction**: Switch between different LLM providers seamlessly
- **API Compatibility**: Drop-in replacement for OpenAI's API

## Key Features

- **= Prompt Versioning**: Track changes and improvements to prompts over time
- **=Ý Template Variables**: Dynamic system and user prompts with Liquid templating
- **=Ê Prompt Evaluation**: Create test sets and measure prompt performance
- **= Provider Integration**: Support for multiple LLM providers with a unified API
- **= API Key Management**: Generate and manage API keys for secure access
- **< OpenAI Compatible API**: Use with existing OpenAI client libraries

## How It Works

### Prompt Architecture

LLMKit supports three types of prompts:

1. **Static System Prompt**: Basic prompt with fixed system instructions
   - Great for simple chat interfaces
   - No dynamic content needed

2. **Dynamic System Prompt**: System prompt with variable substitution
   - Variables are inserted into the system prompt
   - User messages can be free-form

3. **Dynamic System & User Prompts**: Both system and user templates
   - Both system and user prompts can contain variables
   - Ideal for structured inputs with consistent format

![Prompt Architecture](docs/images/prompt-types.png) <!-- [SCREENSHOT PLACEHOLDER] -->

### Template Variables

LLMKit uses a powerful templating system based on Liquid syntax:

#### Variable Substitution
```
You are a helpful assistant named {{ assistant_name }}.
The user's name is {{ user_name }}.
```

#### Conditional Logic
```
{% if formal_tone %}
Please maintain a professional tone in your responses.
{% else %}
Feel free to be casual and friendly.
{% endif %}
```

#### Loops
```
Here are the topics to discuss:
{% for topic in topics %}
- {{ topic }}
{% endfor %}
```

### JSON-based Input

To populate template variables, you pass a JSON object as the system or user message:

```javascript
// System message with context variables
const systemMessage = {
  "role": "system",
  "content": JSON.stringify({
    "assistant_name": "Alex",
    "user_name": "Jordan",
    "formal_tone": true,
    "topics": ["AI", "Machine Learning", "Natural Language Processing"]
  })
}
```

This JSON is automatically parsed and applied to the template, making your prompts dynamic and adaptable.

### OpenAI Compatibility

LLMKit provides 100% compatible API endpoints matching OpenAI's API:

- **Standard API**: `/v1/chat/completions`
- **Streaming API**: `/v1/chat/completions/stream`

This means you can use any OpenAI client library with LLMKit:

```python
from openai import OpenAI

# Point to LLMKit server
client = OpenAI(
    api_key="llmkit_yourkey",
    base_url="http://localhost:8000/v1",
)

# Use like normal OpenAI client
response = client.chat.completions.create(
    model="YOUR-PROMPT-KEY",  # Use your LLMKit prompt key as the model name
    messages=[
        {"role": "system", "content": '{"name": "Alex", "expertise": "AI"}'},
        {"role": "user", "content": "Tell me about machine learning"}
    ]
)
```

### Prompt Evaluation

LLMKit's evaluation system allows you to:

1. Create evaluation test sets with specific inputs
2. Run those inputs against different prompt versions
3. Score and compare performance
4. Track improvements over time

![Evaluation Dashboard](docs/images/eval-dashboard.png) <!-- [SCREENSHOT PLACEHOLDER] -->

## Technical Stack

### Backend

- **Language**: Rust
- **Web Framework**: Actix Web
- **Database**: SQLite with SQLx for type-safe queries
- **Templating Engine**: Liquid templates

### Frontend

- **Framework**: Vue.js (Nuxt.js)
- **Styling**: Tailwind CSS
- **State Management**: Vue Composables

## Setup and Installation

### Backend Setup

#### Prerequisites

- Rust toolchain (latest stable)
- SQLx CLI for database management
- SQLite

#### Installation

1. Install SQLx CLI:
```bash
cargo install sqlx-cli
```

2. Set the database URL:
```bash
export DATABASE_URL="sqlite:llmkit.db"
```

3. Create the database and run migrations:
```bash
cd backend
sqlx database create
sqlx migrate run
```

4. Start the server:
```bash
cargo run
```

The server will start on `http://localhost:8000`.

### Frontend Setup

#### Prerequisites

- Node.js 16+ or Bun

#### Installation

1. Install dependencies:
```bash
cd ui
npm install  # or bun install
```

2. Start the development server:
```bash
npm run dev  # or bun run dev
```

The UI will be available at `http://localhost:3000`.

## Usage

### Managing Prompts

![Prompt Management](docs/images/prompt-management.png) <!-- [SCREENSHOT PLACEHOLDER] -->

1. Create a new prompt from the Prompts page
2. Select a prompt type (static, dynamic system, or dynamic both)
3. Enter your prompt template with variables as needed
4. Choose the LLM provider and model
5. Configure parameters like temperature and max tokens
6. Save and test your prompt

### Testing Prompts

![Prompt Testing](docs/images/prompt-testing.png) <!-- [SCREENSHOT PLACEHOLDER] -->

For chat prompts:
1. Open the prompt testing interface
2. Enter user messages and see the responses
3. For dynamic prompts, provide JSON for variable substitution

For one-shot prompts:
1. Enter the input text or JSON structure
2. Submit the test request
3. View the response and performance metrics

### API Integration

Use your prompts in external applications:

1. Generate an API key from the Settings page
2. Use the OpenAI client library for your programming language
3. Point the client to your LLMKit server
4. Use your prompt keys as model names

Example (Python):
```python
from openai import OpenAI

client = OpenAI(
    api_key="llmkit_your_api_key",
    base_url="http://your_llmkit_server:8000/v1",
)

response = client.chat.completions.create(
    model="YOUR-PROMPT-KEY",
    messages=[
        {"role": "user", "content": "Hello!"}
    ]
)
```

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

[MIT License](LICENSE)