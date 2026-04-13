# agentic-sync

Rust CLI that syncs Claude Code project config to Cursor and Copilot. Single binary, no runtime dependencies.

## Stack

- Rust 2024 edition
- clap 4 for CLI arg parsing (derive API)
- serde + serde_json + serde_yaml for serialization
- similar for text diffing (--pr mode)
- tempfile for test fixtures

## Architecture

Pipeline: discover → parse → IR → generate → output.

- `src/discover.rs` — finds source files (CLAUDE.md, .claude/rules/, .claude/skills/, .mcp.json)
- `src/parse/` — parsers for each source type, produces `ProjectConfig` IR
- `src/ir.rs` — intermediate representation: `ProjectConfig`, `Section`, `Skill`, `McpConfig`
- `src/generate/` — one module per target (cursor.rs, copilot.rs), transforms IR to output files
- `src/output.rs` — file writing, ownership checking (`generated-by` marker), cleanup
- `src/log.rs` — stderr logging with GitHub Actions annotation support

Adding a new target means adding a generator module and an `--out` variant. Parsers don't change.

## Development Commands

- `cargo build` — build debug binary
- `cargo build --release` — build release binary
- `cargo test` — run all unit + integration tests
- `cargo run -- --fix` — generate target files for this repo
- `cargo run -- --check` — verify target files are in sync
- `cargo run -- --pr` — output markdown diff summary
- `hk check -a` — run all linters (requires `mise install`)
- `hk fix -a` — auto-fix all lintable issues

## Code Style

- `cargo fmt` before committing — enforced by hk pre-commit hook
- `cargo clippy -- -D warnings` — zero warnings policy
- No `unwrap()` in library code (src/lib.rs and modules). `unwrap()` is fine in tests and main.rs.
- Prefer `if let` / `let else` over match for single-arm patterns.
- Errors as strings (`Result<T, String>`) for simplicity — no custom error types yet.

## Testing

- TDD: write failing test first, then implement
- Unit tests are inline (`#[cfg(test)] mod tests`) in each module
- Integration tests in `tests/integration/` exercise the full `run()` function
- Use `tempfile::tempdir()` for filesystem tests — never write to the real project dir in tests
- Test both the happy path and edge cases (empty input, missing files, conflicts)

## Conventions

- Commit messages follow conventional commits: `feat:`, `fix:`, `test:`, `chore:`, `docs:`, `refactor:`
- One concern per commit — don't mix features with refactors
- Generated files are always committed (not gitignored)
- All generated files include `generated-by: agentic-sync` in frontmatter for ownership tracking
