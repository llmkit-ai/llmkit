# llmkit Documentation

Welcome to **llmkit** – a modern, extensible platform for managing, testing, and deploying LLM prompts with a focus on versioning, evaluation, and developer-friendly workflows.

---

## Table of Contents
- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Setup & Installation](#setup--installation)
- [Prompt Management](#prompt-management)
- [API & Usage](#api--usage)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

---

## Overview
llmkit provides a unified toolkit for:
- Crafting dynamic prompts with modern template syntax
- Managing, versioning, and evaluating prompts
- Integrating with multiple LLM providers (OpenAI, Azure, OpenRouter, and more)
- Running and tracing prompt executions
- OpenAI-compatible API endpoints

---

## Key Features
- **Prompt Directories & Components**: Organize prompts in folders and reuse prompt parts. Example usage:

  ```
  {{component:component_name}}
  ```

- **Prompt Versioning**: Track changes and roll back as needed
- **Prompt Evaluation**: Create test sets, run evals, and score performance
- **OpenAI-Compatible API**: Use with any OpenAI client library
- **Provider Abstraction**: Unified API for OpenAI, Azure, OpenRouter, and more
- **Modern UI/UX**: Built with Nuxt 3 (Vue 3) and Tailwind CSS

---

## Architecture
- **Backend**: Rust (Axum), SQLite (SQLx), Tera templates
- **Frontend**: Nuxt 3 (Vue 3), Tailwind CSS
- **Docs**: VuePress (Markdown)

### Code Structure
- `backend/` – API, DB, business logic
- `ui/` – Frontend app
- `docs/` – Documentation
- `tests/oai-libs/` – OpenAI client library examples

---

## Setup & Installation

### Quick Start
```sh
# Clone the repo
git clone https://github.com/your-org/llmkit.git
cd llmkit

# Backend
cd backend && cargo build && cargo run

# Frontend
cd ../ui && npm install && npm run dev
```

### Docker
```sh
docker-compose up -d
```

---

## Prompt Management
- **Prompt Types**: Static, dynamic (with variables), and fully templated system/user prompts
- **Template Syntax**: Jinja-style (e.g. `{{ variable }}`, `{% if ... %}`, `{% for ... %}`)
- **Prompt Components**: Reuse prompt parts with:
  ```
  {{component:component_name}}
  ```
- **Prompt Directories**: Organize prompts in folders for easy management

---

## API & Usage
- **RESTful API** for prompt CRUD, execution, and evaluation
- **OpenAI-Compatible Endpoints**:
  - `/v1/chat/completions`
  - `/v1/chat/completions/stream`
- **Sample Prompt Format**:
  ```
  <!-- role:system -->
  you are a helpful assistant
  <!-- role:user -->
  sup dude
  ```
- **Provider Setup**: Configure API keys and base URLs in `.env` and the UI

---

## Development
- **Backend**: Rust, Axum, SQLx, Tera
- **Frontend**: Nuxt 3, Vue 3, Tailwind CSS
- **Docs**: VuePress (see this folder)

### Backend
- See `backend/README.md` for database setup, migrations, and running the server

### Frontend
- See `ui/README.md` for Nuxt app setup and commands

### Tests & Examples
- See `tests/oai-libs/` for OpenAI client usage examples in Python and Node.js

---

## Contributing
- Fork, branch, and open a PR
- See [contributing.md](./contributing.md) for guidelines

---

## License
[MIT License](../LICENSE)
