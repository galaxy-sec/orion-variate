# Repository Guidelines

## Project Structure & Modules
- `src/`: library code. Key modules: `addr/` (HTTP/Git/local access, redirect), `tpl/` (templating), `vars/` (variable resolution), `update/` (download/update ops), `archive.rs`, `timeout.rs`, `types.rs`, `tools.rs`.
- `tests/`: integration tests (e.g., `git_test.rs`, `http_test.rs`) with fixtures in `tests/data/`.
- `examples/`: runnable examples (e.g., `compress_demo.rs`).
- `docs/`: usage and design notes (e.g., `redirect-rules.md`, `net-access-ctrl-guide.md`).
- `tasks/`: design/engineering notes used during development.

## Build, Test, and Development
- Build: `cargo build` (debug) | `cargo build --release` (optimized).
- Test: `cargo test -- --test-threads=1` (matches CI; avoids flaky network-bound tests).
- Lint/format: `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- Coverage: `cargo llvm-cov --all-features --workspace -- --test-threads=1` (install via `cargo install cargo-llvm-cov`).
- Security audit: `cargo audit` (optional; install via `cargo install cargo-audit`).
- Run example: `cargo run --example compress_demo`.

## Coding Style & Naming
- Format with `rustfmt` (4-space indent, edition from `Cargo.toml`).
- Keep `clippy` clean; warnings are CI failures.
- Naming: modules/files `snake_case`; types/traits/enums `UpperCamelCase`; functions/vars `snake_case`; constants `SCREAMING_SNAKE_CASE`.
- Errors: prefer `thiserror` for custom types; avoid `unwrap/expect` outside tests/examples; return `Result<_, orion_error::Error>` or `anyhow::Result` where appropriate.
- Concurrency: prefer `tokio` primitives for async code.

## Testing Guidelines
- Frameworks: `tokio` async tests, `rstest` for parameterized cases, `mockito` for HTTP mocking.
- Locations: unit tests inline (`mod tests`) and integration tests in `tests/` named `*_test.rs`.
- Network I/O: avoid external calls; use mocks. If unavoidable, respect proxies: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` (see comments in `tests/git_test.rs`).
- Temp files: use `tempfile` and clean up after tests.

## Commit & Pull Request Guidelines
- Commits: imperative, focused, and descriptive (e.g., "Add Git accessor proxy support"); reference issues (`#123`) when relevant.
- Before opening a PR: ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass; include a clear summary, rationale, and usage notes; update `docs/` or `examples/` if APIs change.
- CI mirrors these checks and runs on Linux/macOS with stable/beta toolchains.

## Security & Config Tips
- Never commit secrets. CI uses repository secrets (e.g., `CRATES_IO_TOKEN`, `GITHUB_TOKEN`).
- For local proxy-enabled development or tests, set `HTTP(S)_PROXY`/`ALL_PROXY` as needed.
