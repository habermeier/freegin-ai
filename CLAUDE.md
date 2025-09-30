# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based cost-optimized abstraction layer for interacting with multiple generative AI services (OpenAI, Google Gemini, Anthropic Claude, Hugging Face, Cohere). The system provides intelligent routing, budget management, resilience, and extensibility for programmatic code generation.

**Key Technologies**: Rust, Tokio, Axum (web server), SQLx (SQLite database), reqwest (HTTP client)

## Development Commands

### Building & Running
```bash
make build              # Debug build
make release            # Optimized release build
make run                # Start HTTP service
cargo run               # Alternative to make run
```

### Testing & Code Quality
```bash
make test               # Run full test suite
cargo test              # Alternative test command
make fmt                # Format code with rustfmt
cargo clippy            # Run linter (extremely strict lint config)
```

### Installation
```bash
./scripts/bootstrap.sh            # One-command setup (installs deps, builds, installs binary)
make install                       # Install to ~/.local (or PREFIX=<path> make install)
```

### CLI Commands
```bash
# Credential management
freegin-ai add-service huggingface
freegin-ai remove-service huggingface
freegin-ai list-services

# Model catalog management
freegin-ai refresh-models [--provider <name>] [--workload <type>] [--dry-run]
freegin-ai list-models [--provider <name>] [--workload <type>] [--include-suggestions]
freegin-ai adopt-model <provider> <model> [--workload <type>] [--priority <num>]

# Generation
freegin-ai generate [--prompt "text" | --prompt-file <path>] \
  [--context-file <path>] [--output-file <path>] \
  [--complexity low|medium|high] [--quality standard|balanced|premium] \
  [--speed fast|normal] [--guardrail strict|lenient] \
  [--format text|markdown|json] [--provider <name>] [--model <name>]
```

## Architecture

### Core Components

**`src/main.rs`** - Entry point: CLI parsing, logging init, configuration loading, server startup
**`src/lib.rs`** - Module exports for library reuse

**Configuration & Persistence:**
- `src/config.rs` - App configuration with XDG directory support (`~/.config/freegin-ai/`)
- `src/credentials.rs` - Encrypted credential storage using ChaCha20-Poly1305
- `src/database.rs` - SQLite initialization and schema management
- `src/catalog.rs` - Model catalog storage (tracks available models, priorities, workloads)
- `src/usage.rs` - Usage logging for cost tracking and performance analysis

**Request Processing:**
- `src/models.rs` - Core data structures (`AIRequest`, `AIResponse`, request hints, workload types)
- `src/routes.rs` - Axum HTTP route handlers
- `src/error.rs` - Unified error types with `thiserror`

**Provider System:**
- `src/providers/mod.rs` - `AIProvider` trait and `Provider` enum (OpenAI, Google, HuggingFace, Anthropic, Cohere)
- `src/providers/router.rs` - **ProviderRouter**: Intelligent routing logic, fallback handling, model catalog integration
- `src/providers/hugging_face.rs` - Hugging Face client implementation
- `src/providers/google.rs` - Google Gemini client
- `src/providers/openai.rs` - OpenAI client (stub)

### Key Architectural Patterns

**Intelligent Routing**: `ProviderRouter` selects providers based on:
1. Request hints (complexity, quality, speed, guardrails)
2. Model catalog entries (priority, workload suitability)
3. Cached performance metrics (stored in SQLite via `UsageLogger`)
4. Provider availability and authentication status

**Provider Trait Pattern**: All providers implement the `AIProvider` trait (`async fn generate(&self, request: &AIRequest) -> Result<AIResponse, AppError>`), ensuring consistent interfaces.

**Credential Management**: Two-tier system:
1. Config file credentials (`~/.config/freegin-ai/config.toml`)
2. Encrypted database storage (via `CredentialStore`)

**Model Catalog System**: New feature for discovering and prioritizing models per workload (Chat, Code, Summarization, Extraction, Creative, Classification). Providers implement `list_models()` to populate the catalog.

### Request Flow
1. HTTP request arrives at Axum routes (`src/routes.rs`)
2. Request parsed into `AIRequest` structure
3. `ProviderRouter::generate()` selects optimal provider/model
4. Provider-specific client makes API call
5. Response normalized to `AIResponse`
6. Usage logged to SQLite
7. Response returned to client

## Code Standards (Strict Enforcement)

### Documentation Requirements
- **All public items** must have doc comments (`///` or `/** */`)
- Include examples in doc comments using fenced ```rust blocks
- Examples must be runnable via `cargo test`
- Internal comments focus on **why**, not **what**

### Naming Conventions
- Rust files: `snake_case.rs` (e.g., `api_routes.rs`)
- Module entry points: `mod.rs`
- Documentation files: `kebab-case.md` with numeric prefixes (e.g., `docs/01-architecture-overview.md`)

### Testing Requirements
- Every public function must have unit tests
- Integration tests in top-level `tests/` directory
- Tests must be deterministic and avoid external dependencies (use mocks)
- Place tests in same file with `#[cfg(test)] mod tests { ... }`

### Linting
The project enforces **extremely strict** linting via `Cargo.toml`:
- `unsafe_code = "forbid"` - No unsafe code allowed
- `missing_docs = "warn"` - All public items must be documented
- Clippy: `all`, `pedantic`, `nursery`, `cargo` lints enabled
- Some pedantic lints disabled: `module-name-repetitions`, `wildcard-imports`, `disallowed-methods`

## Adding a New Provider

1. Create `src/providers/<provider_name>.rs`
2. Implement `AIProvider` trait for the client struct
3. Add variant to `Provider` enum in `src/providers/mod.rs`
4. Update `Provider::from_alias()` and `Provider::as_str()` methods
5. Register provider in `ProviderRouter::from_config()` (src/providers/router.rs:48)
6. Add credential handling in `handle_add_service()` and `handle_remove_service()` (src/main.rs:600)
7. Update `handle_list_services()` to include the new provider (src/main.rs:641)
8. Implement `list_models()` if provider supports model discovery

## Configuration

**Config file locations** (checked in order):
1. `~/.config/freegin-ai/config.toml` (preferred, XDG standard)
2. `~/.freegin-ai/config.toml` (legacy)
3. `freegin-ai.toml` (project-local)
4. `.secrets/app.toml` (project-local secrets)

**Environment overrides**: Prefix with `APP__` (e.g., `APP__SERVER__PORT=9090`)

**Database**: `~/.local/share/freegin-ai/app.db` by default (override via `DATABASE_URL`)

## Important Notes

- The project uses `sqlx` with compile-time query verification (requires `sqlx-cli`)
- Database schema is managed via `database::ensure_schema()` (not migrations yet)
- Encrypted credentials use ChaCha20-Poly1305 with per-installation keys
- Model catalog system (`src/catalog.rs`) is a recent addition for dynamic model management
- Provider routing considers request "hints" (complexity, quality, speed, guardrail) to select appropriate models