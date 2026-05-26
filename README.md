# USB Stash

> **Your portable encrypted vault. Drop it on a USB drive, run it on any PC. Zero install.**

[![Version](https://img.shields.io/badge/version-0.1.4-blue)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-Apache%202.0-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://rust-lang.org)

USB Stash encrypts your files with **XChaCha20-Poly1305** and stores them in a
single portable container. Carry it on a USB drive. Open it with a double-click.
No installation. No internet connection. No servers. Just you and your password.

---

## Quick install

1. Go to [Releases](https://github.com/jorgealonsodev/usb-stash/releases) and download the files for your OS
2. Copy all downloaded files to your USB drive
3. Run the app:

| OS | Files | Instructions |
|----|-------|-------------|
| **Windows** | `usb-stash-gui-win.exe` + `usb-stash-win.exe` | Double-click `usb-stash-gui-win.exe` for the GUI, or use `usb-stash-win.exe` in terminal |
| **Linux** | `usb-stash-gui-linux` + `usb-stash-linux` + `run.sh` | `bash run.sh` — launches GUI if available, falls back to CLI |

> First run creates a `stash.dat` vault next to the binary.  
> Subsequent runs detect it and prompt for your password.

---

## What it does

| Does | Doesn't |
|------|---------|
| Encrypts files with XChaCha20-Poly1305 (AEAD) | No internet connection |
| Protects passwords with Argon2id (64 MB) | No temp files |
| Detects any file tampering | No installation required |
| Works on Windows 10+ and Linux | No servers or accounts |
| Zeroizes keys and data on close | No password recovery |

---

## Stack

| Layer | Technology | Why |
|------|------------|-----|
| Crypto | `argon2` + `chacha20poly1305` | Modern, audited, no AES-NI required |
| Backend | Rust + Tauri 2 | Small binaries, memory safety, cross-platform |
| Frontend | Svelte 4 + TypeScript | Minimal bundle, native reactivity |
| Build | Cargo workspace + Vite | Monorepo, single `cargo build` |

```
┌──────────────────────────────────────────┐
│              USB DRIVE                   │
│  usbstash-win.exe / usbstash-linux       │
│  stash.dat  (encrypted vault)            │
│  stash.meta (public parameters)          │
└──────────────┬───────────────────────────┘
               ▼
┌──────────────────────────────────────────┐
│         TAURI APP (USB Stash)            │
│  ┌─ Svelte Frontend ──────────────────┐  │
│  │  Login · Create · Explorer · Preview│  │
│  │  Settings · Auto-lock · Export     │  │
│  └──────────┬──────────────────────────┘  │
│             │ Tauri IPC                   │
│  ┌──────────▼──────────────────────────┐  │
│  │  Rust Backend (usbstash-core)       │  │
│  │  Argon2id · AEAD · Format · Stash   │  │
│  └─────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

---

## Usage

### End user

| OS | Instructions |
|----|-------------|
| **Windows** | Double-click `usb-stash-gui-win.exe` (GUI) or use `usb-stash-win.exe` in terminal (CLI) |
| **Linux** | `bash run.sh` (auto-detects GUI/CLI, works on FAT32/exFAT) |

### CLI (developers)

```bash
usbstash create /path/to/stash          # Create new vault
usbstash add /path/to/stash doc.pdf     # Add a file
usbstash list /path/to/stash            # List contents
usbstash extract /path/to/stash doc.pdf # Extract a file
```

---

## Development

### Prerequisites

| Dependency | Version | Install |
|------------|---------|---------|
| Rust | 1.75+ | `rustup install stable` |
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |
| pnpm | — | `npm install -g pnpm` |
| webkit2gtk (Linux) | 4.1-dev | `sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev` |
| WebView2 (Windows) | — | Included in Win 10+ |

### Commands

```bash
cargo build                                # Backend
cd frontend && pnpm install && pnpm build   # Frontend
cargo tauri build                          # Full app
./scripts/build-portable.sh                # USB distribution (Linux)
```

### Tests & Quality

```bash
cargo test --all                         # 155 Rust tests
cd frontend && pnpm test                 # 41 Svelte tests
cargo clippy --all -- -D warnings        # Lint
cargo fmt --check                        # Format
```

---

## Performance

| Operation | Time | Hardware |
|-----------|------|----------|
| Key derivation (Argon2id) | ~600ms | Ryzen 5 5600X |
| Open 1 MB stash | ~650ms | 32 GB RAM |
| Open 100 MB stash | ~800ms | NVMe SSD |
| Encrypt 100 MB | ~180ms | — |

*Benchmarked with [Criterion](https://github.com/bheisler/criterion.rs).*

---

## Documentation

| Document | For |
|----------|-----|
| [FILE_FORMAT.md](FILE_FORMAT.md) | Developers implementing an independent decoder |
| [THREAT_MODEL.md](THREAT_MODEL.md) | Security auditors, pentesters |
| [SECURITY.md](SECURITY.md) | Finding a vulnerability |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contributing code |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

---

## Design decisions

| Decision | Why |
|----------|-----|
| XChaCha20 over AES-GCM | 24-byte nonces (no collision risk), no AES-NI required |
| Argon2id with 64 MB | OWASP 2024: balance of security and interactive usability |
| Single container (`stash.dat`) | Individual filenames never exposed |
| Zero network connections | Defense in depth: the app cannot leak data |
| Svelte over React | Smaller bundle, fewer abstractions |
| Bincode over JSON | Binary, compact, deterministic |

---

## License

Apache 2.0 — [LICENSE](LICENSE)
