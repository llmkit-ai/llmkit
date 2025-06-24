# CLAUDE.md

## Project Overview
Llmkit is a Rust-based LLM prompt management toolkit with a Vue.js frontend. It provides dynamic templating, evaluation, versioning, and OpenAI-compatible APIs.

## Development Commands

### Backend (Rust)
```bash
cd backend
cargo run          # Start development server
cargo test         # Run integration tests
cargo check        # Quick compilation check
cargo clippy       # Rust linting (recommended)
sqlx migrate run   # Run database migrations
```

### Frontend (Vue.js/Nuxt)
```bash
cd ui
bun install        # Install dependencies (preferred)
bun run dev        # Start development server
bun run build      # Build for production
npm run lint       # Run linting (no config currently)
npm run typecheck  # TypeScript type checking (via Nuxt)
```

### Full Stack Development
```bash
docker-compose up -d  # Start both services with Docker
```

## Architecture Overview

### Backend Architecture (Rust + Axum)
- **Layered Architecture:**
  - `controllers/` - HTTP handlers and routing
  - `services/` - Business logic and external integrations
  - `db/` - Data access layer with SQLx type-safe queries
  - `middleware/` - Authentication and request processing
  - `common/` - Shared types and utilities

- **Provider Pattern:** Abstracted LLM provider interface supporting OpenAI, OpenRouter, Azure, with unified service facade
- **Database:** SQLite with migration-based schema management and foreign key constraints
- **Error Handling:** Custom error types with `From` trait implementations for comprehensive error conversion

### Frontend Architecture (Vue.js/Nuxt 3)
- **Composition API** with composables-based state management
- **Feature-based component organization:** `/evals`, `/prompts`, `/tools`
- **Composables:** `usePrompts`, `useModels`, `useProviders` for reusable logic
- **Layouts:** Shared UI structures with authenticated/public layouts

## Coding Standards

### Rust Conventions
- **Naming:** snake_case for functions/variables, PascalCase for structs/enums
- **Error Handling:** Always use custom error types, never unwrap in production code
- **Async Patterns:** Use `#[tokio::test]` for async tests, proper error propagation
- **Database:** All schema changes via versioned SQL migrations, use SQLx macros for type safety

### Frontend Conventions  
- **Naming:** camelCase for JS/TS, PascalCase for components, kebab-case for files
- **Components:** Single-file Vue components with TypeScript
- **State Management:** Prefer composables over global state stores

### Testing Patterns
- **Backend:** Integration tests with in-memory SQLite, test data factories, proper cleanup
- **Database:** Each test gets isolated database instance with full migration run
- **Setup:** Use helper functions for creating test data with UUIDs for uniqueness

## Project Structure
```
backend/           # Rust API server (Axum + SQLite)
├── src/
│   ├── controllers/   # HTTP handlers
│   ├── services/      # Business logic & LLM providers
│   ├── db/           # Data access layer
│   └── middleware/   # Auth & request processing
├── migrations/       # Database schema versions
└── tests/           # Integration tests

ui/               # Vue.js/Nuxt frontend
├── components/      # Feature-organized Vue components
├── composables/     # Reusable logic
├── pages/          # Route-based components
└── types/          # TypeScript definitions
```

## Environment Setup
1. Copy `.env.example` to `.env`
2. Configure required variables:
   - `JWT_SECRET` - Secure random string for authentication
   - `DATABASE_URL` - SQLite database path
   - Provider API keys (`OPENROUTER_API_KEY`, `OPENAI_API_KEY`, etc.)

## Key Technologies
- **Backend:** Rust 2021, Axum web framework, SQLite with SQLx, Tera templating
- **Frontend:** Vue.js 3, Nuxt.js, TypeScript, Tailwind CSS
- **Build:** Cargo for Rust, Bun for frontend (preferred over npm)
- **Deployment:** Docker with multi-stage builds, Docker Compose orchestration
- **Caching:** Moka cache for prompt data with TTL
- **External APIs:** Retry logic with exponential backoff for LLM providers

## Important Notes
- SQLx requires database to exist before compilation (offline mode prepared)
- All LLM providers implement common interface in `services/providers/`
- Frontend uses JetBrains Mono font family (configured in Tailwind)
- Authentication uses JWT tokens with secure cookie handling
- Database migrations must be run before starting backend