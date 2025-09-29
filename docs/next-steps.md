# Next Steps: CLI and Routing Enhancements

1. **Add Non-Interactive CLI Command**  
   - Introduce `freegin-ai generate` subcommand.  
   - Support stdin/stdout by default with `--prompt`, `--prompt-file`, `--output-file`.  
   - Allow context files (`--context-file`) and metadata tags (`--metadata key=value`).

2. **Routing Hint Parameters**  
   - Accept soft-hint flags: `--complexity`, `--quality`, `--speed`, `--guardrail`, `--tags`.  
   - Parse into request metadata passed to the provider router.

3. **Provider Router & Response Handling**  
   - Update `/api/v1/generate` handler to call the router and return actual provider output.  
   - Extend router to consider hints and usage logs when selecting providers, while still allowing manual overrides (`--provider`, `--model`).

4. **Usage Tracking & Reporting Hooks**  
   - Ensure usage logger captures hint metadata for future analytics.  
   - Prepare for `freegin-ai stats` (future work) by structuring the usage table queries.

5. **Documentation & Testing**  
   - Update README, man page, and CLI help to cover the new subcommand and flags.  
   - Add integration tests for the CLI (mock providers) and run full `cargo fmt`, `cargo test`, and `cargo build --release` before committing.
