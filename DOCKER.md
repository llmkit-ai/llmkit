# Docker Deployment for LLMKit

This document provides details about deploying LLMKit using Docker.

## Quick Start

1. Copy the sample environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit the `.env` file and add your:
   - JWT_SECRET (required)
   - API keys for LLM providers (optional, but needed to use those providers)

3. Start the containers:
   ```bash
   docker-compose up -d
   ```

4. Access the application:
   - UI: http://localhost:3000
   - API: http://localhost:8000

## Configuration

### Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| JWT_SECRET | Secret key for JWT token generation | Yes |
| OPENAI_API_KEY | OpenAI API key | No |
| ANTHROPIC_API_KEY | Anthropic API key | No |
| GOOGLE_API_KEY | Google Gemini API key | No |
| DEEPSEEK_API_KEY | DeepSeek API key | No |
| AZURE_ENDPOINT | Azure OpenAI endpoint URL | No |
| AZURE_API_KEY | Azure OpenAI API key | No |

## Container Details

The deployment consists of two containers:

1. **Backend (llmkit-backend)**
   - Rust-based API server
   - Exposed on port 8000
   - Handles API requests, database operations, and LLM provider communication

2. **UI (llmkit-ui)**
   - Nuxt.js frontend
   - Exposed on port 3000
   - Provides the web interface for managing prompts, evaluations, and settings

## Data Persistence

The SQLite database is stored as a volume mount from the host system. By default, it's located at:
```
./backend/llmkit.db
```

This ensures that your data persists between container restarts and updates.

## Common Tasks

### View Logs

```bash
# View logs for all containers
docker-compose logs

# View logs for a specific container
docker-compose logs backend
docker-compose logs ui

# Follow logs in real-time
docker-compose logs -f
```

### Rebuild Containers

If you make changes to the codebase, you'll need to rebuild the containers:

```bash
docker-compose up -d --build
```

### Stop Containers

```bash
docker-compose down
```

## Troubleshooting

### Cannot Connect to Backend

If the UI cannot connect to the backend:

1. Check if the backend container is running:
   ```bash
   docker-compose ps
   ```

2. Check backend logs for errors:
   ```bash
   docker-compose logs backend
   ```

3. Verify that the API_BASE_URL in the UI service is correctly set to `http://backend:8000`

### Database Issues

If you encounter database issues:

1. The database is stored in a Docker volume named `db_data`. You can inspect it with:
   ```bash
   docker volume inspect llmkit-rs_db_data
   ```

2. The application automatically handles database creation and migrations on startup. 
   If you need to reset the database completely, you can remove the volume:
   ```bash
   docker-compose down
   docker volume rm llmkit-rs_db_data
   docker-compose up -d
   ```

   This will create a fresh database with the initial schema applied.