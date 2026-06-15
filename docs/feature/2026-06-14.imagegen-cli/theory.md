# Codex-Authenticated Image CLI

## Thesis

Codex already has the hard part: account auth, provider resolution, and an image API client that knows the Codex backend endpoints. The standalone CLI should stay as a thin companion, exposing the full image request surface while delegating authentication and HTTP semantics to Codex crates.

The CLI should not invoke the model-facing `image_gen.imagegen` tool. That tool is tied to agent turn context, history, and event emitters. A direct CLI has no turn lifecycle, so the right boundary is `codex-api::ImagesClient`.

## Strategy

The project is independent from the Codex source tree and depends on Codex crates through pinned git dependencies. This keeps `Cargo.toml` portable and avoids local absolute paths. Local path overrides can be used outside version control if iteration speed becomes important.

The implementation avoids `codex-core`. It parses only the Codex config fields needed for auth reuse: model, credential store mode, forced workspace IDs, ChatGPT base URL, and OpenAI provider base URL. `codex-login::AuthManager` remains the source of truth for auth loading. `codex-model-provider` resolves the provider and auth provider passed to Codex API clients.

Initially the plan was to call `codex-api::ImagesClient` directly for all real work. Live Codex OAuth requests to the direct images endpoint return 404, matching public reports that Codex OAuth image generation currently routes through `/backend-api/codex/responses` with the hosted `image_generation` tool. The CLI therefore exposes an explicit transport choice: `codex-hosted` by default for out-of-box Codex auth, and `image-api` for custom/future direct images endpoints. Both transports receive the same user-facing option set where supported.

The CLI exposes image options directly: model, background, quality, size, n, and transport. Edit accepts repeated `--image` flags and converts local PNG/JPEG/WebP files to data URLs. The image count cap follows Codex's extension guardrail of five images.

## Verification

Automated coverage focuses on parsing and request mapping because those are the local responsibilities. Network behavior is verified by a manual smoke test that performs a low-quality `codex-hosted` generation through Codex auth and writes a PNG.
