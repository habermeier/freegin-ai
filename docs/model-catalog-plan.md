# Model Catalog Refresh Plan

## Goals

- Maintain a catalogue of candidate models per provider and workload (e.g., chat, summarisation, code).
- Automate discovery of new models by prompting existing providers (“refresh” command).
- Allow manual review/adoption of suggested models; track the status of each candidate.
- Keep usage metrics (latency, success) for adopted models to inform future refreshes.

## Data Model

- `provider_models` table: active models by provider/workload, including metadata such as default hints.
- `provider_model_suggestions` table: pending ideas with fields `(provider, workload, model, rationale, status, created_at, updated_at)`.
- Extend `provider_usage` logging to include optional `model` so we can score performance.

## CLI Additions

- `freegin-ai refresh-models [provider]` — aggregates current catalog + usage metrics, prompts the router for recommendations, stores results in `provider_model_suggestions`.
- `freegin-ai list-models [--provider] [--workload]` — shows active models alongside suggestions and usage stats.
- `freegin-ai adopt-model <provider> <model> [--workload WORKLOAD]` — moves a suggestion into the active catalog (or updates an existing entry).
- `freegin-ai retire-model <provider> <model>` (optional) — marks a model inactive.

## Provider Prompt Template

Compile context with:
- Provider name, current active models by workload, usage statistics (avg latency/success, last used).
- Constraints (cost/latency targets) derived from configuration.
- Instruction prompt requesting a JSON response per workload: `{ "workload": "chat", "recommendations": [ { "model": "...", "reason": "...", "cost": "...", "speed": "..." } ] }`.

Call the router using a high-quality model (e.g., existing Hugging Face default). Parse JSON; insert records as `status='pending'`.

## Routing Integration

- When selecting providers, load a `Vec<ModelCandidate>` per provider/workload ordered by status (adopted first, pending optional).
- Respect manual overrides (`--provider`, `--model`), otherwise pick from active catalog using hints.
- If catalog is empty, fall back to config defaults. If config defaults missing, rely on suggestions.

## Testing Strategy

- Unit tests for catalog CRUD operations (using in-memory SQLite).
- Integration tests for CLI commands with mocked prompts (e.g., pre-defined suggestion payload).
- Update README/man page to describe new commands and model-catalog flow.
