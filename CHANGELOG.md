# Changelog — USB Stash

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] — 2026-05-26

### Fixed
- Linux binary on USB drives: `run.sh` launcher copies binary to `/tmp` when filesystem is mounted `noexec` (FAT32/exFAT). Prevents "Permission denied" errors.
- Release workflow now ships the correct standalone CLI binary instead of the Tauri binary which requires bundled frontend assets.

### Added
- `run.sh` launcher script: detects exec availability, copies to temp when needed, cleans up on exit.

## [0.1.0] — 2026-05-26

### Added
- Argon2id key derivation (64 MB memory, 3 iterations, 4 parallelism)
- XChaCha20-Poly1305 AEAD encryption for file contents
- Binary container format (STSH magic, header/chunk/footer structure)
- CLI tool with commands: `create`, `add`, `list`, `extract`
- Tauri desktop application with Svelte frontend
- File explorer with tree view and virtual paths
- In-memory preview for PDF, image, and text files
- Settings panel (password change, auto-lock, export)
- Portable USB distribution scripts (`build-portable.sh`)
- Typed error hierarchy (`CryptoError`, `StashError`, `CliError`)
- Zeroization of sensitive data on lock/exit
- Metadata file (`stash.meta`) with KDF parameters and failed attempt tracking
