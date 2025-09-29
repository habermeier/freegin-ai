# Agent Collaboration & Development Guidelines

This document provides firm instructions for all agents, human or AI, contributing to this project. Adherence to these guidelines is mandatory to ensure code quality, maintainability, and a consistent development experience. A disciplined approach from all contributors is essential for the project's long-term success.

## 1. Documentation & In-Code Comments

All code MUST be thoroughly documented. No exceptions. Code without documentation is considered incomplete.

### 1.1 Public API Documentation (pub fn, pub struct, etc.)

- Provide doc comments for every public item, describing its purpose, parameters, return values, and any potential panics.
- Use `///` for single-line doc comments and `/** ... */` for multi-line blocks.
- Include code examples using fenced ```rust blocks wherever they clarify usage. Examples must be self-contained and runnable via `cargo test` so documentation never becomes stale or incorrect.

### 1.2 Internal Logic Comments

- Use `//` comments only for non-obvious logic; avoid restating self-explanatory code.
- Focus comments on the **why**, not the **what**, explaining algorithmic choices or business rules.
- Prefer precise guidance such as `// Iterate over providers in priority order to find one that is available` instead of generic statements.
- Document complex algorithms, state-management logic, unconventional approaches, and potential gotchas that might surprise another developer.

## 2. Naming Conventions

Consistency is key to readability and maintainability.

### 2.1 Rust Files (`.rs`)

- Follow standard Rust snake_case file naming (e.g., `api_routes.rs` for API route handlers).
- When a module lives in a directory, name the entry-point file `mod.rs` (e.g., `src/providers/mod.rs`).

### 2.2 Documentation Files (`.md`, `.txt`)

- All documentation within `docs/` must use `kebab-case.md` filenames for consistency and URL friendliness.
- Prefix documentation files with an ordering number (e.g., `docs/01-getting-started.md`) to provide a guided reading path for new contributors.

## 3. Testing

Untested code is considered broken code; rigorous testing is mandatory.

- Every public function must have a corresponding unit test covering expected behavior, edge cases, and error conditions.
- Refactor complex or critical private functions into smaller, pure functions when needed so they can be tested in isolation.
- Place integration tests that verify interactions between modules in the top-level `tests/` directory.
- Use Rust's `#[test]` attribute and built-in testing framework.
- Keep tests self-contained and independent of external services by using mocks, stubs, or test doubles. This ensures tests remain fast, deterministic, and suitable for offline CI/CD pipelines.

## 4. AI Provider Architecture

The system is designed for modularity, allowing new AI providers to be added or updated with minimal friction.

### 4.1 Adding a New Provider

- Create a new module in `src/providers/` (e.g., `src/providers/anthropic.rs`).
- Implement the `AIProvider` trait for the provider's client struct to guarantee a consistent interface.
- Add the provider to the `Provider` enum in `src/providers/mod.rs`.
- Update the factory function in `src/providers/mod.rs` to construct the new provider client, ensuring it is registered and available to the rest of the application.
