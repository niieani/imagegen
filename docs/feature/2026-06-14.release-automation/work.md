# Work

## Release Flow

- CI on push/PR: fmt, clippy, test on macOS and Linux.
- Release Please on `main`: conventional commits -> release PR -> GitHub release.
- Manifest starts at `0.0.0` so the first release PR publishes `0.1.0`.
- Release publish on GitHub release/manual dispatch:
  - build Apple Silicon macOS and Linux release binaries
  - sign/notarize macOS binaries
  - upload tarballs and checksums
  - load Homebrew tap token from 1Password
  - update `niieani/homebrew-tap` cask

## Artifacts

- `imagegen_<version>_darwin_arm64.tar.gz`
- `imagegen_<version>_linux_amd64.tar.gz`
- `imagegen_<version>_linux_arm64.tar.gz`
- `checksums.txt`

## Required Secret

- `OP_SERVICE_ACCOUNT_TOKEN`: required before first release publish.
