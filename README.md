# imagegen

Standalone Rust CLI companion for Codex-authenticated OpenAI image generation.

`imagegen` reuses Codex config in `CODEX_HOME` or `~/.codex`. With the default
OpenAI provider, it uses the Codex-hosted Responses image generation transport;
with a custom `model_provider`, it routes through that provider's direct Images
API config.

## Install

### Homebrew

After the first GitHub release is published:

```sh
brew tap niieani/tap
brew install --cask imagegen
```

### Download with GitHub CLI

Download the latest release for the current platform:

```sh
set -euo pipefail
case "$(uname -s)-$(uname -m)" in
  Darwin-x86_64)
    echo "darwin_amd64 releases are not published; build from source" >&2
    exit 1
    ;;
  Darwin-arm64) asset="darwin_arm64" ;;
  Linux-x86_64) asset="linux_amd64" ;;
  Linux-aarch64 | Linux-arm64) asset="linux_arm64" ;;
  *) echo "unsupported platform: $(uname -s)-$(uname -m)" >&2; exit 1 ;;
esac

tmp="$(mktemp -d)"
gh release download --repo niieani/imagegen --pattern "imagegen_*_${asset}.tar.gz" --dir "$tmp"
tar -xzf "$tmp"/imagegen_*_"$asset".tar.gz -C "$tmp"
install -m 0755 "$tmp/imagegen" "$HOME/.local/bin/imagegen"
```

Use another install directory if `~/.local/bin` is not on `PATH`.

### Build Locally

Build an optimized binary and refresh the repo-root `./imagegen` symlink:

```sh
just build
```

Then run:

```sh
./imagegen --help
```

The symlink points at `target/release/imagegen`.

To install into Cargo's bin directory instead:

```sh
cargo install --path .
```

Requires a recent Rust toolchain with edition 2024 support.

## Release Automation

Releases are managed by Release Please. When a release is published, GitHub
Actions builds macOS and Linux archives, notarizes macOS binaries, uploads
checksums, and updates `niieani/homebrew-tap`.

Published archives cover Apple Silicon macOS plus Linux amd64/arm64. Intel
macOS users should build from source.

## Authentication

Log in with Codex first:

```sh
codex login
```

The Codex-hosted transport hard-fails if Codex ChatGPT/backend authentication is
unavailable. It does not silently fall back to `OPENAI_API_KEY`.

For custom providers, configure Codex as usual:

```toml
model_provider = "custom"

[model_providers.custom]
name = "Custom Images"
base_url = "https://images.example.com/v1"
env_key = "CUSTOM_IMAGE_API_KEY"
wire_api = "responses"
supports_websockets = false
```

## Generate

```sh
imagegen generate \
  --prompt "a small blue ceramic teapot on a white table" \
  --out teapot.png
```

Default transport is provider-aware. The built-in OpenAI provider uses
`codex-hosted`, which uses Codex OAuth against the Responses API image
generation tool and supports one output image per request. Custom
`model_provider` entries use `image-api` by default. Pass
`--transport codex-hosted` to force Codex-hosted generation when Codex login is
available.

Default image model is `gpt-image-2`. It supports `auto` or constrained
`WIDTHxHEIGHT` sizes, including common sizes such as `1024x1024`,
`1536x1024`, `1024x1536`, `2048x2048`, `2048x1152`, `3840x2160`, and
`2160x3840`.

## Edit

Pass one or more input images with repeated `--image` flags. Inputs can be
edited directly or used as references for a new image; make the prompt specify
what to do with them and what to preserve:

```sh
imagegen edit \
  --image input-1.png \
  --image input-2.webp \
  --prompt "use these inputs as references and combine them into one product photo" \
  --out combined.png
```

At most 5 input images are accepted.

## Options

Both `generate` and `edit` expose:

- `--prompt`
- `--out` `.png` output path; parent directories are created as needed
- `--model` default `gpt-image-2`
- `--background` one of `auto`, `opaque`, `transparent`; `gpt-image-2` does
  not support `transparent`
- `--quality` one of `auto`, `low`, `medium`, `high`; use `low` for drafts
- `--size` default `auto`; for `gpt-image-2`, custom sizes must have max edge
  `<=3840px`, both edges divisible by 16, aspect ratio `<=3:1`, and total
  pixels between `655360` and `8294400`
- `--n`
- `--transport` one of `codex-hosted`, `image-api`

`edit` additionally requires one or more repeated `--image` paths. Supported
input formats: PNG, JPEG, WebP.

For `codex-hosted`, omit `--n` or set `--n 1`. The direct `image-api` transport
passes `--n` through to the Images API request.

Large, complex, high-quality, or 4K `gpt-image-2` requests can take up to about
2 minutes. For text-heavy images such as screenshots, labels, posters, or
newspapers, include ALL required text in the prompt. `gpt-image-2` is very
likely to render prompt-provided text correctly.
