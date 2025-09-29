# Architecture Overview

This document summarises the goals and guiding principles behind the Generative AI Service Abstraction Layer. It complements the high-level README and dives deeper into the non-functional requirements that shape the system design.

## Motivation

Teams that rely on multiple generative AI providers wrestle with inconsistent APIs, pricing schemes, quotas, and reliability profiles. The abstraction layer consolidates those concerns behind a single service so application developers can focus on their product logic instead of provider minutiae.

## Core Capabilities

- **Intelligent routing** chooses the best provider for each request by consulting cached performance metrics (latency, error rate) and current cost posture.
- **Budget governance** enforces spend limits through a circuit breaker that halts paid requests once thresholds are crossed while allowing free-tier fallbacks to continue.
- **Resilience engineering** adds exponential back-off, structured retries, and automatic failover when a provider is degraded or offline.
- **Provider extensibility** keeps integration effort low by requiring only an implementation of the shared `AIProvider` trait for new vendors; the current prototype ships with Hugging Face as the default integration path.
- **Usage learning and caching** uses SQLite to log calls, cache deterministic responses, and inform future routing/budget decisions with empirical data collected via the in-process usage logger.

## High-Level Components

| Component | Responsibility |
| --- | --- |
| API Gateway (Axum) | Receives HTTP requests, authenticates clients, and shapes responses |
| Request Router | Evaluates provider health, pricing, and past performance to pick a target |
| Provider Connectors | Translate generic requests into provider-specific payloads and parse responses |
| Usage & Cost Ledger | Persists call metadata, balances, and historical metrics in SQLite |
| Feedback Loop | Collects human signals about output quality to refine routing heuristics |

## Deployment Considerations

- Ships as a single Rust binary with zero unsafe code allowed (`unsafe_code = "forbid"`).
- Works with SQLite out of the box; can evolve to Postgres with minimal change thanks to `sqlx`.
- Integrates with CI via GitHub Actions to run formatting, clippy, and test suites on every push.
- Persists operator configuration under `~/.config/freegin-ai/` following XDG conventions, with environment overrides for runtime adjustments.

Use this overview alongside `02-directory-structure.md` to understand where each concern lands in the repository.
