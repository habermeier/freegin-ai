# Model Catalog System Guide

The model catalog system provides intelligent, automated management of AI models across providers and workload types.

## Overview

The catalog maintains:
- **Active roster**: Models currently in use, ordered by priority
- **Suggestions**: Candidate models discovered via automated refresh
- **Usage statistics**: Performance metrics for intelligent ranking
- **Default models**: Automatically seeded bootstrap configuration

## Core Concepts

### Workload Types

Models are organized by workload category:

- **Chat**: General conversation and Q&A
- **Code**: Code generation and analysis
- **Summarization**: Text summarization tasks
- **Extraction**: Information extraction from text
- **Creative**: Creative writing and ideation
- **Classification**: Text classification and labeling

### Model Status

**Active roster** (`provider_models` table):
- `active`: Currently in use
- `retired`: Previously used, kept for reference

**Suggestions** (`provider_model_suggestions` table):
- `pending`: Awaiting human review
- `trial`: Under evaluation (future feature)
- `adopted`: Moved to active roster

### Priority System

Lower numbers = higher priority (10 > 100 > 1000)

The router selects the highest-priority (lowest number) active model for each provider/workload combination.

## CLI Commands

### List Models

Show active models for a provider/workload:

```bash
# All active models
freegin-ai list-models

# Filter by provider
freegin-ai list-models --provider huggingface

# Filter by workload
freegin-ai list-models --workload chat

# Include suggestions
freegin-ai list-models --provider huggingface --workload chat --include-suggestions
```

**Output format:**
```
Provider: huggingface | Workload: Chat
  Active:
     50  mistralai/Mistral-7B-Instruct-v0.2 — General-purpose chat model
    100  tiiuae/falcon-7b-instruct — Fallback option

  Suggestions:
    pending  meta-llama/Llama-2-7b-chat-hf — Better reasoning capabilities
```

### Adopt a Model

Move a model into the active roster:

```bash
freegin-ai adopt-model huggingface "mistralai/Mistral-7B-Instruct-v0.2" \
  --workload chat \
  --priority 50
```

**Parameters:**
- `<provider>`: Provider name (huggingface, google, openai, etc.)
- `<model>`: Model identifier (e.g., `mistralai/Mistral-7B-Instruct-v0.2`)
- `--workload`: Workload type (chat, code, summarization, etc.)
- `--priority`: Priority number (default: 100)

**Output:**
```
Adopted model 'mistralai/Mistral-7B-Instruct-v0.2' for provider 'huggingface' and workload 'Chat' with priority 50

Active models for huggingface / Chat:
   50  mistralai/Mistral-7B-Instruct-v0.2 —
  100  tiiuae/falcon-7b-instruct —
```

### Refresh Model Suggestions

Query an LLM to discover new candidate models:

```bash
# Refresh suggestions for a specific provider/workload
freegin-ai refresh-models --provider huggingface --workload chat

# Preview without writing to database
freegin-ai refresh-models --provider huggingface --workload chat --dry-run

# Refresh all workloads for a provider (if no workload specified, defaults to Chat)
freegin-ai refresh-models --provider huggingface
```

**How it works:**

1. Collects current roster and usage statistics
2. Builds context JSON with performance data
3. Calls a reliable LLM with structured prompt
4. Parses JSON response with model suggestions
5. Inserts suggestions into `provider_model_suggestions` table

**Example context sent to LLM:**
```json
{
  "provider": "huggingface",
  "workload": "Chat",
  "current_models": [
    {
      "model": "mistralai/Mistral-7B-Instruct-v0.2",
      "priority": 50,
      "rationale": "General-purpose chat model"
    }
  ],
  "usage_stats": {
    "total_calls": 147,
    "success_rate": 98.6,
    "avg_latency_ms": 842.3
  }
}
```

**Example LLM response:**
```json
{
  "suggestions": [
    {
      "model": "meta-llama/Llama-2-7b-chat-hf",
      "workload": "Chat",
      "rationale": "Better reasoning and context handling than current models",
      "production_ready": true,
      "notes": "Requires API key, slightly higher latency",
      "metadata": {"est_cost_per_1k_tokens": 0.20}
    }
  ]
}
```

**Dry-run output:**
```
=== DRY RUN MODE ===
Would insert 3 suggestions:

1. meta-llama/Llama-2-7b-chat-hf (Chat)
   Rationale: Better reasoning and context handling
   Production ready: true
   Notes: Requires API key, slightly higher latency

2. ...
```

## Database Schema

### `provider_models` (Active Roster)

```sql
CREATE TABLE provider_models (
    provider TEXT NOT NULL,        -- Provider name (huggingface, google, etc.)
    workload TEXT NOT NULL,        -- Workload type (chat, code, etc.)
    model TEXT NOT NULL,           -- Model identifier
    status TEXT NOT NULL,          -- 'active' or 'retired'
    priority INTEGER NOT NULL,     -- Lower = higher priority
    rationale TEXT,                -- Human-readable reason for selection
    metadata TEXT,                 -- JSON with additional info
    created_at TEXT NOT NULL,      -- RFC3339 timestamp
    updated_at TEXT NOT NULL,      -- RFC3339 timestamp
    UNIQUE(provider, workload, model)
);
```

### `provider_model_suggestions` (Candidates)

```sql
CREATE TABLE provider_model_suggestions (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    workload TEXT NOT NULL,
    model TEXT NOT NULL,
    status TEXT NOT NULL,          -- 'pending', 'trial', 'adopted'
    rationale TEXT,                -- Why this model was suggested
    metadata TEXT,                 -- JSON with cost estimates, etc.
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(provider, workload, model)
);
```

### `provider_usage` (Performance Tracking)

```sql
CREATE TABLE provider_usage (
    id INTEGER PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT,                    -- Model used (NULL for legacy entries)
    success INTEGER NOT NULL,      -- 1 = success, 0 = failure
    latency_ms INTEGER NOT NULL,   -- Request latency
    error_message TEXT,            -- Error details if failed
    prompt_tokens INTEGER,         -- Token counts (future)
    completion_tokens INTEGER,
    total_tokens INTEGER,
    input_cost_micros INTEGER,     -- Cost tracking (future)
    output_cost_micros INTEGER,
    total_cost_micros INTEGER,
    created_at TEXT NOT NULL
);
```

### Indexes

```sql
CREATE INDEX idx_provider_models_active
  ON provider_models(provider, workload, status, priority);

CREATE INDEX idx_provider_model_suggestions
  ON provider_model_suggestions(provider, workload, status);

CREATE INDEX idx_provider_usage_provider_model_time
  ON provider_usage(provider, model, created_at);
```

## Default Models

On first run, the system automatically seeds these defaults for HuggingFace:

| Workload | Model | Priority | Rationale |
|----------|-------|----------|-----------|
| Chat | mistralai/Mistral-7B-Instruct-v0.2 | 100 | General-purpose chat model |
| Code | bigcode/starcoder | 100 | Specialized for code generation |
| Summarization | facebook/bart-large-cnn | 100 | Optimized for summarization |
| Extraction | tiiuae/falcon-7b-instruct | 100 | Good for information extraction |
| Creative | mistralai/Mistral-7B-Instruct-v0.2 | 100 | Creative writing and ideation |
| Classification | facebook/bart-large-mnli | 100 | Text classification and NLI |

Defaults are only seeded if no active models exist for that provider/workload.

## Router Integration

The `ProviderRouter` automatically selects models using this logic:

1. If `request.model` is specified → use that
2. If `request.hints.provider` is specified → use that (legacy)
3. Otherwise → query catalog for highest-priority active model matching provider/workload
4. Log usage with chosen model for future optimization

**Code location**: `src/providers/router.rs:137` (`generate` method)

## Usage Statistics

The catalog can aggregate usage data for decision-making:

```rust
let stats = catalog.usage_stats(Provider::HuggingFace, Some(Workload::Chat)).await?;

println!("Total calls: {}", stats.total_calls);
println!("Success rate: {:.1}%", stats.success_rate);
println!("Avg latency: {:.0}ms", stats.avg_latency_ms);
```

This data is passed to the LLM during `refresh-models` to inform suggestions.

## Migration from Existing Databases

The system automatically migrates older databases:

1. Detects missing `model` column in `provider_usage`
2. Adds `model TEXT` column via `ALTER TABLE`
3. Adds cost tracking columns if missing
4. Creates indexes if they don't exist

**Code location**: `src/database.rs:181` (`migrate_provider_usage_columns`)

## Workflow Example

### Initial Setup

```bash
# 1. Add provider credentials (if needed)
freegin-ai add-service huggingface

# 2. View default models
freegin-ai list-models --provider huggingface

# 3. Adopt a higher-priority model for chat
freegin-ai adopt-model huggingface "mistralai/Mistral-7B-Instruct-v0.2" \
  --workload chat \
  --priority 50
```

### Periodic Refresh

```bash
# 1. Refresh suggestions (weekly/monthly)
freegin-ai refresh-models --provider huggingface --workload chat

# 2. Review suggestions
freegin-ai list-models --provider huggingface --workload chat --include-suggestions

# 3. Adopt promising candidates
freegin-ai adopt-model huggingface "meta-llama/Llama-2-7b-chat-hf" \
  --workload chat \
  --priority 40

# 4. Test the new model via generate
echo "Tell me a joke" | freegin-ai generate --provider huggingface --workload chat
```

### Production Operations

```bash
# Monitor active roster
freegin-ai list-models --provider huggingface

# Retire underperforming models (manual DB update for now)
sqlite3 ~/.local/share/freegin-ai/app.db \
  "UPDATE provider_models SET status='retired' WHERE model='old-model'"

# Check usage statistics (future: built-in command)
sqlite3 ~/.local/share/freegin-ai/app.db \
  "SELECT model, COUNT(*), AVG(latency_ms), SUM(success)*100.0/COUNT(*) as success_rate
   FROM provider_usage
   WHERE provider='huggingface'
   GROUP BY model"
```

## Future Enhancements

**Planned features:**

1. **Automated trials**: Set suggestions to `status='trial'` and route a percentage of traffic
2. **Cost tracking**: Populate token/cost fields and rank by cost-per-success
3. **Smoke tests**: `freegin-ai smoke --provider X --workload Y` to validate models
4. **Retire command**: `freegin-ai retire-model <provider> <model> --workload <type>`
5. **Stats command**: `freegin-ai stats --provider X --workload Y`
6. **Nightly refresh**: Cron job or systemd timer for automated discovery
7. **Multi-provider fallback**: Rank models across providers for intelligent failover

## Troubleshooting

**No models returned by router:**
- Check: `freegin-ai list-models --provider <name> --workload <type>`
- Verify defaults were seeded (should happen automatically)
- Adopt a model manually if needed

**Refresh fails with "No available AI provider":**
- Ensure you have valid credentials: `freegin-ai list-services`
- Add credentials: `freegin-ai add-service <provider>`

**Database errors about missing columns:**
- Migration should run automatically on startup
- Verify with: `sqlite3 ~/.local/share/freegin-ai/app.db ".schema provider_usage"`
- Check logs for migration errors

**Suggestions not appearing:**
- Run with `--dry-run` first to see what would be inserted
- Check LLM response format (must be valid JSON matching schema)
- Verify suggestions weren't filtered out (invalid workload names)

## Code References

- **Catalog store**: `src/catalog.rs`
- **CLI handlers**: `src/main.rs:819-1100`
- **Router integration**: `src/providers/router.rs:137,289`
- **Database schema**: `src/database.rs:69-136`
- **Usage logging**: `src/usage.rs`