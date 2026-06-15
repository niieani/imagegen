# AGENTS.md

- Owner: Bazyli (niieani).
- Response style: concise, telegraph.
- This is a standalone Rust CLI companion to Codex; keep it small.
- Reuse Codex crates by pinned git dependency. Do not commit local absolute path dependencies.
- Auth must reuse Codex login and hard-fail when missing; no silent `OPENAI_API_KEY` fallback.
- Expose all image API options supported by the Codex image client.
- Prefer tests for CLI parsing and request mapping before implementation changes.
- Run `cargo fmt` and `cargo test` after code changes.
- Put scratch files under `temp.local/` and do not commit them.
