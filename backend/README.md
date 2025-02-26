# LLMKit Backend

The LLMKit backend is built with Rust using Actix Web for the HTTP server and SQLite for persistence.

## Features

- RESTful API for prompt management and execution
- OpenAI-compatible endpoints
- Provider abstraction layer for multiple LLM services
- SQLite database for storage with SQLx for type-safe queries
- Prompt versioning and evaluation

## Setup and Development

### Prerequisites

- Rust toolchain (latest stable)
- SQLx CLI for database management
- SQLite

### Installation

1. Install SQLx CLI if you don't have it:
```bash
cargo install sqlx-cli
```

2. Set the database URL:
```bash
export DATABASE_URL="sqlite:llmkit.db"
```

### Database Setup

1. Create a new database:
```bash
sqlx database create
```

2. Run migrations to set up the schema:
```bash
sqlx migrate run
```

### Creating New Migrations

1. Create a new migration file:
```bash
sqlx migrate add <migration_name>
```

2. Edit the generated SQL file in the `migrations` directory

3. Run the migration:
```bash
sqlx migrate run
```

4. Prepare SQLx metadata (for offline compile-time checking):
```bash
cargo sqlx prepare --check
```

### Running the Server

```bash
cargo run
```

The server will start on `http://localhost:8000` by default.

## API Endpoints

### Prompts

- `GET /v1/ui/prompts` - List all prompts
- `POST /v1/ui/prompts` - Create a new prompt
- `GET /v1/ui/prompts/{id}` - Get a specific prompt
- `PUT /v1/ui/prompts/{id}` - Update a prompt
- `DELETE /v1/ui/prompts/{id}` - Delete a prompt

### Prompt Execution

- `POST /v1/ui/prompts/execute/{id}` - Execute a prompt
- `POST /v1/ui/prompts/execute/{id}/stream` - Stream a prompt execution
- `POST /v1/ui/prompts/execute/chat` - Execute a prompt in chat mode

### OpenAI-Compatible Endpoints

- `POST /v1/chat/completions` - Chat completions API
- `POST /v1/chat/completions/stream` - Streaming chat completions

### API Keys

- `GET /v1/ui/settings/api-keys` - List API keys
- `POST /v1/ui/settings/api-keys` - Create a new API key
- `DELETE /v1/ui/settings/api-keys/{id}` - Delete an API key

## Code Structure

- `src/controllers/` - API route handlers
- `src/db/` - Database access layer
- `src/middleware/` - HTTP middleware (auth, etc.)
- `src/services/` - Business logic and LLM provider integrations

## Sample Prompt Format

```
<!-- role:system -->
you are a helpful assistant

<!-- role:user -->
sup dude
```