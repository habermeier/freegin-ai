# Generative AI Service Abstraction Layer

This project provides a cost-optimized, intelligent abstraction layer for interacting with various generative AI services. It is designed to be a robust, resilient, and scalable platform for programmatic code generation, based on the principles outlined in the accompanying architecture blueprint.

## Core Features

- **Intelligent routing** dynamically selects the best AI provider based on cost, performance, and real-time availability.
- **Cost management** enforces strict budget controls with a circuit breaker to prevent overspending.
- **Resilience** handles API errors, rate limits, and service unavailability with exponential back-off and automatic failover.
- **Extensibility** allows new AI providers to be added by implementing a common `AIProvider` trait.
- **Learning & caching** uses a local SQLite database to cache responses and learn provider performance characteristics over time.

## Getting Started

### One-Command Bootstrap

Run the bundled script to install missing prerequisites (Rust toolchain, GNU Make, `sqlx-cli`) and place the binary on your shell path:

```bash
./scripts/bootstrap.sh
```

This script is idempotent and safe to rerun; it delegates to your platform package manager when it needs to install `make`, installs `sqlx-cli` via Cargo if necessary, seeds `~/.config/freegin-ai/config.toml`, and copies the compiled `freegin-ai` binary to `~/.local/bin` by default (emitting a reminder if that directory is not on your `PATH`). Use `./scripts/bootstrap.sh --system` or `--prefix DIR` to change the installation destination.

### Manual Setup

1. **Install Rust** with `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
2. **Install `sqlx-cli`** using `cargo install sqlx-cli` for compile-time checked queries.
3. **Configure secrets** by copying `.config/template.toml` to `.secrets/app.toml` and filling in provider keys (the `.secrets` directory is ignored by Git).
4. **Create a `.env` file** if you need overrides (the default database lives under `~/.local/share/freegin-ai/app.db` so this step is optional).
5. **Ensure GNU Make is available** (macOS/Linux usually ship it; Windows users can rely on WSL or MSYS2).
6. **Use the Makefile helpers** (feel free to set `PREFIX` or `DESTDIR` to control installation):
   - `make build` — compile the project without running it.
   - `make release` — produce an optimised release binary in `target/release/freegin-ai`.
   - `make run` — start the HTTP service using your current configuration.
   - `make test` — execute the full test suite.
- `make install` — copy the release binary and man page to `$(PREFIX)/bin` and `$(PREFIX)/share/man/man1` (defaults to `~/.local`).

The server binds to the host and port defined in your configuration when launched via `make run` or `freegin-ai`.

### Configuration Layout

- User-specific configuration lives at `~/.config/freegin-ai/config.toml` (auto-created by the bootstrap script if missing).
- Legacy support for `~/.freegin-ai/config.toml` and project-local overrides (`freegin-ai.toml`, `.secrets/app.toml`) remains in place for developers.
- Environment variables prefixed with `APP__` override individual settings (e.g. `APP__SERVER__PORT=9090`).
- Provider sections are optional; uncomment the ones you need. Example:

  ```toml
  [providers.hugging_face]
  api_key = "hf_api_token"
  api_base_url = "https://api-inference.huggingface.co"
  ```

  Add `tags: ["provider:hf"]` to a request payload (or reference a Hugging Face model name such as `org/model`) to route to that provider explicitly.

### Managing Provider Tokens

- Use the CLI helper to store encrypted credentials locally:

  ```bash
  freegin-ai add-service huggingface
  ```

  The command prints the Hugging Face token URL, prompts for your key (input hidden), and saves it encrypted in the local SQLite database. You can rerun the command to rotate the token at any time.
- Remove a stored token with `freegin-ai remove-service huggingface`, or inspect which providers are configured/stored via `freegin-ai list-services`.

### Manual Installation Targets

- Binaries: `$(PREFIX)/bin/freegin-ai` (`PREFIX` defaults to `~/.local`).
- Manual page: `$(PREFIX)/share/man/man1/freegin-ai.1`.
- Data directory (database, usage logs): `~/.local/share/freegin-ai/` by default (overridable via `DATABASE_URL`).

Once installed you can consult the manual page via `man freegin-ai` or run `freegin-ai --help` for a quick summary (the bootstrap script prints this output automatically).

## Project Goals

- Provide a sustainable pipeline for low-cost, high-quality code generation using multiple AI providers.
- Maintain strong operational visibility through usage tracking and budgeting guardrails.
- Offer a developer-friendly API with clear documentation and strict linting/testing standards.

Refer to `AGENTS.md` for contribution expectations and `docs/` for deeper architectural references and design documents.
