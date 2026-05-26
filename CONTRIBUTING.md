# Contributing to USB Stash

Thank you for your interest in contributing to USB Stash! This document provides
guidelines and instructions for contributing.

## Code of Conduct

Be respectful, constructive, and inclusive. We welcome contributors of all
experience levels.

## How to Contribute

### Reporting Bugs

1. Check the [issue tracker](https://github.com/usbstash/usbstash/issues) to
   see if the bug has already been reported.
2. If not, open a new issue with:
   - A clear title and description
   - Steps to reproduce
   - Expected vs. actual behavior
   - Environment (OS, Rust version, USB Stash version)

### Suggesting Features

1. Open an issue with the `enhancement` label.
2. Describe the feature, its use case, and why it would be valuable.
3. Discuss the approach before implementing — we want to align on design first.

### Security Issues

**Do NOT open a public issue.** See [SECURITY.md](SECURITY.md) for responsible
disclosure procedures.

### Pull Requests

1. **Fork** the repository and create your branch from `main`.
2. **Create a feature branch** with a descriptive name:
   - `feat/add-totp-support`
   - `fix/zeroize-on-lock`
   - `docs/update-threat-model`
3. **Write tests** for new functionality. Run `cargo test --all` before pushing.
4. **Follow the coding style:**
   - `cargo fmt` — code is formatted with `rustfmt`
   - `cargo clippy -- -D warnings` — no clippy warnings allowed
5. **Write clear commit messages** using [conventional commits](https://www.conventionalcommits.org/):
   - `feat: add TOTP support to settings`
   - `fix: zeroize master key on lock`
   - `docs: expand threat model with side-channel analysis`
6. **Keep PRs focused.** One logical change per PR. If your change is large,
   consider splitting it into smaller, reviewable PRs.
7. **Update documentation** if your change affects user-facing behavior.
8. **Open the PR** with a clear description of what changed and why.

## Development Setup

### Prerequisites

- Rust 1.75+ (`rustup install stable`)
- Node.js 18+ (for frontend)
- pnpm (`npm install -g pnpm`)
- Tauri dependencies (see [Tauri docs](https://v2.tauri.app/start/prerequisites/))

### Build and Test

```bash
# Install dependencies
cargo build --all

# Run all tests
cargo test --all

# Format code
cargo fmt

# Lint
cargo clippy --all -- -D warnings

# Build Tauri app (development)
cd src-tauri && cargo tauri dev
```

### Project Structure

```
usbstash/
├── crates/
│   ├── usbstash-core/    # Pure Rust library (crypto, format, stash API)
│   └── usbstash-cli/     # CLI binary
├── src-tauri/            # Tauri desktop app backend
├── frontend/             # Svelte frontend
└── scripts/              # Build and distribution scripts
```

### Coding Conventions

- **Error handling:** Use `Result<T, E>` with typed errors. No `unwrap()` or
  `expect()` in production code.
- **Documentation:** Public items must have rustdoc comments.
- **Security:** Never log passwords, keys, or plaintext. Use `zeroize` for
  sensitive data.
- **Naming:** "USB Stash" in user-facing text, `usbstash` in code/identifiers.

## Release Process

Releases are managed by maintainers. The process:

1. Update `CHANGELOG.md` with the new version and changes.
2. Bump version in `Cargo.toml` files.
3. Tag the release: `git tag v0.1.0`
4. Push the tag: `git push origin v0.1.0`
5. GitHub Actions builds and publishes the release.

## Questions?

Open an issue or reach out to the maintainers. We're happy to help!
