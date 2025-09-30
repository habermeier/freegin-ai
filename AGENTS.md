# Agent Collaboration Guidelines

This document provides universal guidance for AI agents collaborating across multiple projects and AI CLI tools. For project-specific technical details, see `CLAUDE.md` (Claude Code), or equivalent files for other AI tools.

## 1. Documentation Standards

All code must be thoroughly documented. No exceptions.

### 1.1 Public API Documentation

- Provide doc comments for every public item, describing purpose, parameters, return values, and potential panics
- Use language-appropriate documentation syntax (e.g., `///` for Rust, `"""` for Python, `/** */` for JavaScript)
- Include runnable examples in documentation where applicable
- Examples must be self-contained and verifiable via the project's test suite

### 1.2 Internal Logic Comments

- Comment only non-obvious logic; avoid restating self-explanatory code
- Focus on **why**, not **what** - explain algorithmic choices and business rules
- Document complex algorithms, state management, unconventional approaches, and potential gotchas
- Prefer precise, specific comments over generic statements

## 2. Naming Conventions

Consistency is critical for readability and maintainability.

### 2.1 Source Code Files

- Follow language idioms: `snake_case` for Rust/Python, `camelCase` for JavaScript, etc.
- Use descriptive names that indicate purpose (e.g., `api_routes.rs`, `userController.js`)

### 2.2 Documentation Files

- All documentation must use `kebab-case` filenames (e.g., `getting-started.md`, `api-reference.md`)
- Prefix with ordering numbers for guided reading paths (e.g., `01-overview.md`, `02-setup.md`)
- Avoid spaces and special characters in filenames

## 3. Testing Requirements

Untested code is considered broken code.

- Every public function must have corresponding unit tests covering expected behavior, edge cases, and error conditions
- Refactor complex private functions into testable units when necessary
- Place integration tests in language-appropriate locations (`tests/` for Rust, `__tests__/` for JavaScript, etc.)
- Tests must be deterministic and independent of external services - use mocks, stubs, or test doubles
- Tests should run fast and work offline for effective CI/CD integration

## 4. Universal Problem-Solving Workflow

When encountering complex technical problems:

1. **Define the problem clearly** - What's broken? What's the expected behavior?
2. **Gather context** - Collect relevant files, logs, configuration, and recent changes
3. **Classify the issue** - Build system, runtime error, performance, testing, etc.
4. **Research solutions** - Check documentation, existing tests, similar patterns in the codebase
5. **Propose approach** - Outline solution before implementing
6. **Implement incrementally** - Small, testable changes
7. **Verify thoroughly** - Run tests, check edge cases, verify no regressions

## 5. Cross-AI Collaboration

When working across multiple AI CLI tools (Claude Code, Gemini CLI, etc.):

### 5.1 File Organization

- **Project-specific instructions**: Keep in root-level files (`CLAUDE.md`, `GEMINI.md`, `AGENTS.md`)
- **Global instructions**: Store in user home directory (`~/.claude/CLAUDE.md`, `~/.gemini/GEMINI.md`)
- **Shared context**: Use `AGENTS.md` for universal guidance applicable to all AI tools

### 5.2 Context Handoffs

When packaging context for another AI tool:
- Use structured problem descriptions
- Include relevant file paths and line numbers
- Provide recent changes (git diffs, commit history)
- Document what's been tried and what failed
- State specific questions or goals clearly

### 5.3 Tool-Specific Features

- Claude Code: Strong at code analysis, architecture understanding, multi-file refactoring
- Gemini CLI: Optimized for specific workflows (check tool documentation)
- Other tools: Respect tool-specific strengths and limitations

## 6. Code Quality Principles

### 6.1 Clarity Over Cleverness

- Write code that's easy to understand, not code that shows off language features
- Optimize for readability first, performance second (unless performance is critical)
- Future maintainers (including you) will thank you

### 6.2 Consistent Style

- Follow project-specific linting rules (see `CLAUDE.md` or equivalent)
- Use automated formatters religiously (rustfmt, prettier, black, etc.)
- Match existing code style when modifying files

### 6.3 Error Handling

- Handle errors explicitly; avoid silent failures
- Provide actionable error messages
- Log enough context to diagnose issues in production

## 7. AI-Specific Considerations

### 7.1 When to Ask for Help

- **Unclear requirements**: Ask for clarification before implementing
- **Architectural decisions**: Propose options, let humans decide on major changes
- **Breaking changes**: Always warn about backward compatibility impacts
- **Security concerns**: Flag potential vulnerabilities immediately

### 7.2 Incremental Progress

- Break large tasks into smaller, verifiable steps
- Commit working code frequently (when appropriate)
- Don't try to fix everything at once

### 7.3 Tool Awareness

- Be aware of your current AI CLI context (Claude Code, Gemini CLI, etc.)
- Know which files contain relevant instructions for your current tool
- Understand what tools and commands are available in your environment

---

**Note**: This file contains universal guidance. For technical details specific to this project (commands, architecture, code organization), refer to tool-specific files like `CLAUDE.md`.
