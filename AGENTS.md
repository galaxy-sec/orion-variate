# Repository Guidelines

This guide helps you contribute effectively to orion_variate. Follow these practices to keep CI green and changes maintainable.

## Project Structure & Module Organization
- `src/`: library code
  - `addr/`: HTTP/Git/local access and redirect rules
  - `tpl/`: templating
  - `vars/`: variable resolution
  - `update/`: download/update operations
  - Common modules: `archive.rs`, `timeout.rs`, `types.rs`, `tools.rs`
- `tests/`: integration tests (`*_test.rs`); fixtures in `tests/data/`
- `examples/`: runnable demos (e.g., `compress_demo.rs`)
- `docs/`: usage/design notes (e.g., `redirect-rules.md`, `net-access-ctrl-guide.md`)
- `tasks/`: engineering notes

## Build, Test, and Development Commands
- Build debug: `cargo build`; release: `cargo build --release`
- Test (CI-identical): `cargo test -- --test-threads=1`
- Format check: `cargo fmt --all -- --check`
- Lint (no warnings): `cargo clippy --all-targets --all-features -- -D warnings`
- Coverage: `cargo llvm-cov --all-features --workspace -- --test-threads=1`
- Security audit (optional): `cargo audit`
- Run example: `cargo run --example compress_demo`

## Coding Style & Naming Conventions
- Rust 4-space indent; edition from `Cargo.toml`; format with `rustfmt`
- Keep `clippy` clean; fix or `#[allow]` with rationale
- Names: modules/files `snake_case`; types/traits/enums `UpperCamelCase`; fns/vars `snake_case`; consts `SCREAMING_SNAKE_CASE`
- Errors: prefer `thiserror`; avoid `unwrap/expect` outside tests/examples; return `Result<_, orion_error::Error>` or `anyhow::Result`
- Concurrency: prefer `tokio` primitives for async

## Testing Guidelines
- Frameworks: `tokio` async tests, `rstest` for params, `mockito` for HTTP
- Keep tests hermetic; mock network. If unavoidable, respect proxies: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`
- Integration tests live in `tests/` and are named `*_test.rs`; fixtures in `tests/data/`
- Use `tempfile`; clean up temporary artifacts

## Commit & Pull Request Guidelines
- Commits: imperative and focused (e.g., "Add Git accessor proxy support"); reference issues `#123`
- Before PR: ensure `cargo fmt`, `cargo clippy`, and `cargo test -- --test-threads=1` pass; update `docs/` or `examples/` if APIs change
- PRs: clear summary, rationale, usage notes; include logs or screenshots if relevant
- CI mirrors local checks on Linux/macOS with stable/beta toolchains

## Security & Configuration Tips
- Never commit secrets; CI uses repository secrets (`CRATES_IO_TOKEN`, `GITHUB_TOKEN`)
- For proxy-enabled development or tests, set `HTTP(S)_PROXY`/`ALL_PROXY` as needed
