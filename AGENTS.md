# proofcraft - AGENTS.md

Pure-Rust ArtCraft repository (`L1` `proofcraft-core`, `L2` `proofcraft-mcp`, `L3` `proofcraft-app`).

## Rules
- Follow `mcp-central-docs/standards/CRAFT_FLEET_RULES.md` and `mcp-central-docs/standards/RUST_ARTCRAFT_STANDARD.md`.
- Single trunk (`main`), LF line endings (`.gitattributes`), zero em dashes anywhere.
- Never commit `target/`, `dist/`, `*.exe`, `*.sqlite`, `.env`, or `*.bak`.
- Clippy lints (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`) are `deny` workspace-wide.
- Verify every change with:
  1. `cargo fmt --all --check`
  2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  3. `cargo test --workspace`
  4. `cargo run -p proofcraft-app -- --smoke-test`
