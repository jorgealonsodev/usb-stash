# Changelog — USB Stash

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **GUI migrated from Tauri/Svelte to egui** — the desktop app is now a single native binary built with `eframe`/`egui`. No Node.js, no webkit, no WebView2 runtime required.
- Replaced Tauri IPC layer with direct `usbstash-core` method calls behind `Arc<Mutex<Option<Stash>>>`.
- File dialogs now use `rfd` (native OS dialogs) instead of Tauri's dialog plugin.
- Auto-lock tracks egui input events via `Instant` tracker (no OS-specific idle detection).

### Removed
- `src-tauri/` directory (Tauri backend, IPC commands, build config)
- `frontend/` directory (Svelte SPA, 11 components, 4 routes)
- Node.js and pnpm from development prerequisites
- `cargo tauri` from build and release workflows

### Added
- `crates/usbstash-gui/` — new egui desktop application with Login, Create, Explorer, and Settings screens.
- Password input widget with reveal toggle.
- Entropy bar widget for password strength feedback.
- Tree view widget (collapsible folder tree from flat entries).
- File table widget (sortable entry list).
- Context menu widget (Extract, Rename, Delete — ready for table row integration).

## [0.2.0] — 2026-05-26

### Changed
- **GUI rewritten in egui** — replaced Tauri/Svelte with pure Rust GUI using eframe + egui for true single-binary portability. No WebView, no Node.js, no npm. Single 15MB executable.
- `src-tauri/` and `frontend/` directories removed from workspace.

### Added
- All 4 screens: Login, Create, Explorer, Settings
- Password strength meter (zxcvbn) on create screen
- Delete confirmation dialog
- `Stash::rename_entry()` method in core
- Auto-lock via egui input event tracking

### Fixed
- Binary size optimized: 28MB → 15MB via LTO + opt-level="z"
- 3 previously-stubbed Tauri commands now fully implemented

## [0.1.4] — 2026-05-26

### Fixed
- Release workflow: replaced `cargo tauri build` (requires cargo subcommand plugin) with `npx tauri build` (uses npm-installed Tauri CLI). Previous attempt failed with `no such command: tauri`.

## [0.1.3] — 2026-05-26

### Fixed
- Tauri GUI binary now embeds frontend assets at compile time using `cargo tauri build` instead of `cargo build`. Previous releases shipped a binary without embedded HTML/CSS/JS, causing "Could not connect to localhost: Connection refused" because the WebView fell back to the Vite dev server URL.

## [0.1.2] — 2026-05-26

### Fixed
- Release now includes the full Tauri GUI app (`usb-stash-gui-linux`, `usb-stash-gui-win.exe`) with embedded frontend assets. Previous release only shipped the CLI binary, causing "Could not connect to localhost" errors when users expected a GUI.

### Added
- Both GUI and CLI binaries shipped per platform. `run.sh` auto-detects and launches GUI first, falling back to CLI.

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
