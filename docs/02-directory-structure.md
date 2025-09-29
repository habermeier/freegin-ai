# Directory Structure

This layout keeps runtime assets, secrets, documentation, and code clearly separated. The tree below shows the intended shape once the project is fully initialised.

```
├── .cargo/              # Cargo configuration for build/linting profiles
│   └── config.toml
├── .config/             # Application configuration templates and defaults
│   └── template.toml
├── .github/
│   └── workflows/
│       └── rust.yml     # CI pipeline (fmt, clippy, tests)
├── data/                # Persistent data, e.g. SQLite database file
├── docs/                # Project documentation (kebab-case, ordered)
│   ├── 01-architecture-overview.md
│   ├── 02-directory-structure.md
│   ├── 2025-09-23-project-overview.md/
│   └── man/
│       └── freegin-ai.1 # Manual page installed alongside binaries
├── src/                 # Application source code (library + binary)
│   ├── config.rs
│   ├── credentials.rs
│   ├── database.rs
│   ├── error.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── models.rs
│   ├── providers/
│   │   ├── google.rs
│   │   ├── hugging_face.rs
│   │   ├── mod.rs
│   │   └── openai.rs
│   └── routes.rs
├── tests/               # Integration tests executed with `cargo test`
│   └── providers.rs
├── .gitignore           # Ignore rules for secrets, build artefacts, etc.
├── AGENTS.md            # Contribution guidelines for humans and AI agents
├── Cargo.toml           # Crate manifest with strict lint profiles
├── Makefile             # Build/test/install wrapper around Cargo
└── README.md            # High-level project overview and setup steps
```

### Notes

- `.secrets/` is intentionally absent; it is ignored by Git and should be created locally to store API keys.
- Add new documentation as `NN-title.md` to preserve the reading order.
- Provider modules live under `src/providers/`; each implements the shared `AIProvider` trait and can be exercised from `tests/`.
