# imagegen

Standalone Rust CLI companion for Codex-authenticated OpenAI image generation.

`imagegen` reuses the Codex login in `CODEX_HOME` or `~/.codex`, then calls the
Codex-hosted Responses image generation transport by default.

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
  Darwin-x86_64) asset="darwin_amd64" ;;
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

The publish workflow requires the `OP_SERVICE_ACCOUNT_TOKEN` repository secret.
Apple notarization credentials and the Homebrew tap token are loaded from
1Password during the workflow.

## Authentication

Log in with Codex first:

```sh
codex login
```

The CLI hard-fails if Codex ChatGPT/backend authentication is unavailable.
It does not silently fall back to `OPENAI_API_KEY`.

## Generate

```sh
imagegen generate \
  --prompt "a small blue ceramic teapot on a white table" \
  --quality low \
  --out teapot.png
```

Default transport is `codex-hosted`, which uses Codex OAuth against the
Responses API image generation tool. It supports one output image per request.
Use `--transport image-api` only for a custom/future direct Images API endpoint;
the current live Codex backend does not expose the direct images endpoint.

## Edit

Pass one or more input images with repeated `--image` flags:

```sh
imagegen edit \
  --image input-1.png \
  --image input-2.webp \
  --prompt "combine these into one product photo" \
  --out combined.png
```

At most 5 input images are accepted.

## Options

Both `generate` and `edit` expose:

- `--prompt`
- `--out`
- `--model` default `gpt-image-2`
- `--background` one of `auto`, `opaque`, `transparent`
- `--quality` one of `auto`, `low`, `medium`, `high`
- `--size` default `auto`
- `--n`
- `--transport` one of `codex-hosted`, `image-api`

`edit` additionally requires one or more `--image` paths. Supported input
formats: PNG, JPEG, WebP.

For `codex-hosted`, omit `--n` or set `--n 1`. The direct `image-api` transport
passes `--n` through to the Images API request.
