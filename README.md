# freegin-ai

**A multi-provider AI gateway with intelligent routing, health tracking, and automatic failover.**

`freegin-ai` is a lightweight abstraction layer that routes AI requests across multiple providers (Groq, DeepSeek, Together AI, Google Gemini, Hugging Face, OpenAI, Anthropic, Cohere). It provides reliable, cost-effective AI access through intelligent provider selection, automatic failover, and comprehensive health tracking.

## Why freegin-ai?

**Problem:** AI providers have rate limits, outages, and varying costs. Managing multiple API keys and handling failures manually is tedious and error-prone.

**Solution:** `freegin-ai` automatically:
- Routes requests to the best available provider based on health status and priorities
- Falls back to alternative providers when one fails or hits rate limits
- Tracks provider health with exponential backoff for temporary failures
- Stores API keys encrypted locally (never committed to version control)
- Provides clean output modes for piping and code generation

**Free Tier Focus:** The default configuration prioritizes providers with generous free tiers:
- **Groq**: 14,400 requests/day, ultra-fast inference (truly free)
- **DeepSeek**: Unlimited free usage with powerful reasoning (truly free)
- **Together AI**: Free tier models after $5 deposit (Llama 3.3 70B)

## Quick Start

### 1. Install

```bash
# Bootstrap installs Rust, builds the binary, and sets up config
./scripts/bootstrap.sh
```

Or install manually:

```bash
cargo build --release
make install  # Installs to ~/.local/bin
```

### 2. Configure Providers (Interactive)

The fastest way to get started is the interactive setup wizard:

```bash
freegin-ai --init
```

This walks you through each provider with sign-up URLs, allowing you to:
- Add API keys for providers you want to use
- Skip providers you don't need (just press Enter)
- Re-run anytime to add more providers

All keys are stored **encrypted** in a local SQLite database at `~/.local/share/freegin-ai/app.db`.

### 3. Test It

```bash
# Let the router choose the best provider
freegin-ai generate --prompt "Hello, world!"

# Check provider status
freegin-ai status

# See which providers are configured
freegin-ai list-services
```

## Usage

### Basic Generation

```bash
# Simple prompt
freegin-ai generate --prompt "Explain recursion in one sentence"

# From file
freegin-ai generate --prompt-file query.txt --output-file response.txt

# With context files
freegin-ai generate --prompt "Review this code" --context-file src/main.rs

# Force a specific provider
freegin-ai generate --prompt "Hello" --provider groq
```

### Output Modes

```bash
# Default: Clean output (content only, perfect for piping)
freegin-ai generate --prompt "Write a Python function" > code.py

# Verbose: Show metadata on stderr
freegin-ai generate --prompt "Hello" --verbose
# Output:
# === Metadata ===
# Provider: groq
#
# === Response ===
# Hello! How can I help you?

# JSON format: Structured output with metadata
freegin-ai generate --prompt "Hello" --format json
# Output: {"provider": "groq", "content": "Hello!..."}

# JSON metadata: Separate metadata stream
freegin-ai generate --prompt "Hello" --emit-metadata
```

### Routing Hints

Influence provider selection with soft hints:

```bash
freegin-ai generate \
  --prompt "Write optimized C++ code" \
  --complexity high \
  --quality premium \
  --speed normal
```

Available hints:
- `--complexity`: `low`, `medium`, `high`
- `--quality`: `standard`, `balanced`, `premium`
- `--speed`: `fast`, `normal`
- `--guardrail`: `strict`, `lenient`

### Provider Management

```bash
# Interactive setup for all providers
freegin-ai --init

# Add a specific provider
freegin-ai add-service groq
freegin-ai add-service deepseek
freegin-ai add-service together

# List configured providers
freegin-ai list-services
# Output:
# groq: stored
# deepseek: stored
# huggingface: none

# Check provider health and active models
freegin-ai status
# Shows health status, consecutive failures, last success, etc.

# Remove provider credentials
freegin-ai remove-service groq
```

## Supported Providers

| Provider | Free Tier | Speed | Best For |
|----------|-----------|-------|----------|
| **Groq** | 14.4K req/day | ⚡ Ultra-fast | Quick queries, high volume |
| **DeepSeek** | Unlimited | Fast | Heavy usage, reasoning |
| **Together AI** | Requires $5 deposit | Fast | Llama 3.3 70B free tier |
| **Google Gemini** | 60 req/min | Fast | Multimodal tasks |
| **Hugging Face** | Rate-limited | Varies | Specialized models |
| **OpenAI** | Pay-as-you-go | Fast | Production workloads |
| **Anthropic** | Limited credits | Fast | Complex reasoning |
| **Cohere** | Free tier | Fast | Experimentation |

See `docs/providers-setup.md` for detailed setup instructions and API key URLs.

## Architecture

### Health Tracking

The health tracking system automatically monitors provider reliability:

- **Error Classification**: Rate limits, auth failures, service outages, transient errors
- **Exponential Backoff**: Automatic retry delays (1min → 2min → 4min → ... → 60min)
- **Status Management**:
  - `Available`: Ready to use
  - `Degraded`: Temporary issues, will retry after backoff period
  - `Unavailable`: Critical failure (auth, out of credits), retry after 24 hours

```bash
# Check provider health
freegin-ai status

# Provider-specific health
freegin-ai status --provider groq
```

### Model Catalog

The model catalog manages workload-specific model selection:

```bash
# List active models
freegin-ai list-models

# List models for specific workload
freegin-ai list-models --workload code

# Add a model to the roster
freegin-ai adopt-model \
  --provider groq \
  --workload code \
  --model llama-3.3-70b-versatile \
  --priority 10

# Discover new models using LLM
freegin-ai refresh-models --provider groq --workload chat
```

Workload types: `chat`, `code`, `summarization`, `extraction`, `creative`, `classification`

### Configuration

Configuration is loaded from (in priority order):

1. Environment variables: `APP__SERVER__HOST`, `APP__SERVER__PORT`, `DATABASE_URL`
2. User config: `~/.config/freegin-ai/config.toml`
3. Legacy locations: `~/.freegin-ai/config.toml`, `freegin-ai.toml`

Example `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
url = "sqlite://~/.local/share/freegin-ai/app.db"

[providers.groq]
api_key = ""  # Leave empty, use encrypted storage instead
api_base_url = "https://api.groq.com/openai/v1"

[providers.deepseek]
api_key = ""
api_base_url = "https://api.deepseek.com"
```

**Best Practice**: Use encrypted credential storage (`freegin-ai add-service`) instead of storing keys in the config file.

## HTTP Server Mode

Run as a persistent HTTP service:

```bash
freegin-ai  # Starts server on configured host:port
```

API endpoint:

```bash
curl -X POST http://localhost:8080/api/v1/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Hello, world!",
    "hints": {
      "complexity": "low",
      "quality": "standard"
    }
  }'
```

## Development

### Build and Test

```bash
# Development build
cargo build

# Run tests
cargo test

# Release build
cargo build --release

# Install locally
make install  # Installs to ~/.local/bin

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt
```

### Project Structure

```
freegin-ai/
├── src/
│   ├── main.rs           # CLI and server entry point
│   ├── lib.rs            # Library exports
│   ├── config.rs         # Configuration management
│   ├── database.rs       # SQLite setup and migrations
│   ├── models.rs         # Core data structures
│   ├── credentials.rs    # Encrypted credential storage
│   ├── health.rs         # Provider health tracking
│   ├── catalog.rs        # Model catalog and workload routing
│   ├── usage.rs          # Usage logging
│   ├── routes.rs         # HTTP API routes
│   └── providers/
│       ├── mod.rs        # Provider trait and enum
│       ├── router.rs     # Intelligent routing logic
│       ├── groq.rs       # Groq client
│       ├── deepseek.rs   # DeepSeek client
│       ├── together.rs   # Together AI client
│       ├── google.rs     # Google Gemini client
│       └── hugging_face.rs  # HuggingFace client
├── docs/
│   ├── providers-setup.md      # Provider setup guide
│   ├── model-catalog-guide.md  # Model catalog documentation
│   └── man/freegin-ai.1        # Man page
└── scripts/
    └── bootstrap.sh      # Installation script
```

### Adding a New Provider

1. Create provider client implementing `AIProvider` trait in `src/providers/`
2. Add provider to `Provider` enum in `src/providers/mod.rs`
3. Add configuration struct to `src/config.rs`
4. Update router in `src/providers/router.rs` to initialize the provider
5. Add default models to `src/catalog.rs` seed function
6. Update `handle_add_service()` and `handle_init()` in `src/main.rs`
7. Add sign-up URL to help text

## Security

- **Encrypted Storage**: API keys stored using ChaCha20-Poly1305 encryption
- **No Keys in Git**: `.gitignore` excludes all credential files
- **Hidden Input**: Password prompts use `rpassword` for hidden terminal input
- **Database Location**: `~/.local/share/freegin-ai/app.db` (user-only permissions)

## Contributing

See `AGENTS.md` for development workflow and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Support

- 📖 Documentation: `freegin-ai --help` or `man freegin-ai`
- 🐛 Issues: [GitHub Issues](https://github.com/habermeier/freegin-ai/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/habermeier/freegin-ai/discussions)