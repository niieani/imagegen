# Work

## Decisions

- New standalone repo.
- Rust edition 2024.
- Public dependencies use pinned git deps to `https://github.com/openai/codex`.
- No committed local path dependencies.
- No model-mediated `image_gen.imagegen` invocation.
- Default transport: Codex-hosted Responses image generation tool.
- Direct API transport: `AuthManager` -> `create_model_provider` -> `ImagesClient`.
- Edit input images use repeated `--image`.
- Max edit images: 5.

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo build`
- Manual low-quality generation:
  `cargo run -- generate --prompt "..." --quality low --out temp.local/YYYY-MM-DD/smoke.png`
