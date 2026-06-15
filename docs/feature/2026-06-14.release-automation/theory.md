# Release Automation Theory

## Thesis

`imagegen` should be usable as a companion binary without requiring users to clone the repo or build Rust dependencies. The release system therefore owns three outcomes: verified source changes, notarized macOS/Linux release archives, and a Homebrew cask path that stays current after each release.

The repo is intentionally small, but the dependency graph comes from Codex git crates. CI should validate the Rust surface directly rather than introducing a release framework that hides the build. Release artifacts are predictable tarballs named by platform and architecture; Homebrew cask URLs are derived from the GitHub release tag.

## Operating Model

Release Please owns version bumps and GitHub Release creation from conventional commits. A separate publish workflow reacts to a published release, builds native release binaries, notarizes macOS artifacts with 1Password-backed Apple credentials, uploads archives/checksums to the release, then updates `niieani/homebrew-tap`.

The 1Password pattern follows `bb-project`: configure the service account from `OP_SERVICE_ACCOUNT_TOKEN`, load Apple metadata, read the `.p8` and `.p12` payloads with `op`, and avoid storing credential material in the repo. The homebrew tap token also comes from 1Password at `op://Automation/GitHub Token for homebrew-tap/token`, so the only GitHub secret needed for release publishing is `OP_SERVICE_ACCOUNT_TOKEN`.

## Boundaries

The first release cannot complete until `OP_SERVICE_ACCOUNT_TOKEN` exists in the GitHub repo secrets. CI and Release Please do not require that secret. The Homebrew tap receives a bootstrap cask locally, but the source repo publish workflow is authoritative for replacing cask checksums with exact release artifact hashes.
