# USB Stash

> Bóveda cifrada portable — viví en un USB, cero instalación.

[![Version](https://img.shields.io/badge/version-0.1.0-blue)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-Apache%202.0-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://rust-lang.org)

**USB Stash** es una aplicación de escritorio portable que vive en un USB y permite almacenar archivos dentro de una bóveda cifrada con **XChaCha20-Poly1305**. Sin instalación. Sin dependencias del sistema host. Criptografía moderna y auditable.

---

## Características

- 🔐 **XChaCha20-Poly1305** AEAD — cifrado autenticado con nonces de 24 bytes
- 🧂 **Argon2id** — key derivation resistente a fuerza bruta GPU (64 MB, 3 iteraciones)
- 📦 **Contenedor único** — todos los archivos en un solo `stash.dat` cifrado
- 🚫 **Cero red** — la app no hace ninguna conexión saliente
- 🧹 **Sin archivos temporales** — todo en memoria, zeroización al cerrar
- 🖥️ **Windows + Linux** — binarios standalone, doble click y listo
- 🎨 **Dark mode** — interfaz minimalista y seria

---

## Arquitectura

```
┌──────────────────────────────────────────────┐
│              USB DRIVE                       │
├──────────────────────────────────────────────┤
│  usbstash-win.exe      (binario Windows)     │
│  usbstash-linux        (binario Linux)       │
│  stash.dat             (stash cifrado)       │
│  stash.meta            (metadata + salt)     │
│  README.txt            (instrucciones)       │
└──────────────────────────────────────────────┘
              │
              ▼
┌──────────────────────────────────────────────┐
│           APLICACIÓN TAURI (USB Stash)       │
│  ┌────────────────────────────────────────┐  │
│  │  Frontend Svelte (WebView del SO)      │  │
│  │  - Login / Create stash                │  │
│  │  - File explorer con vista árbol       │  │
│  │  - Preview PDF / imagen / texto        │  │
│  │  - Settings (contraseña, auto-lock)    │  │
│  └────────────────┬───────────────────────┘  │
│                   │ Tauri IPC                 │
│  ┌────────────────▼───────────────────────┐  │
│  │  Backend Rust                          │  │
│  │  - Crypto (Argon2id + AEAD)            │  │
│  │  - Binary format (STSH container)      │  │
│  │  - In-memory FS + auto-save            │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

---

## Uso rápido

### Windows
1. Copiá `usbstash-win.exe` a tu USB
2. Doble click → creá o abrí tu stash
3. Arrastrá archivos, previsualizá, exportá

### Linux
```bash
chmod +x usbstash-linux
./usbstash-linux
```

### CLI (opcional)
```bash
usbstash create /ruta/stash
usbstash add /ruta/stash documento.pdf
usbstash list /ruta/stash
usbstash extract /ruta/stash documento.pdf
```

---

## Desarrollo

### Prerrequisitos

- **Rust** 1.75+
- **Node.js** 18+ / pnpm
- **Linux:** `libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev`
- **Windows:** WebView2 (incluido en Win 10+)

### Build

```bash
# Backend
cargo build

# Frontend
cd frontend && pnpm install && pnpm build

# App completa
cargo tauri build

# Distribución portable (USB)
./scripts/build-portable.sh   # Linux
.\scripts\build-portable.ps1  # Windows
```

### Tests

```bash
cargo test --all        # 155 tests Rust
cd frontend && pnpm test  # 41 tests Svelte/Vitest
cargo clippy --all -- -D warnings
cargo fmt --check
```

---

## Benchmarks

| Operación | Tamaño | Tiempo |
|-----------|--------|--------|
| KDF (Argon2id) | — | ~600ms |
| Cifrado | 1 MB | ~2ms |
| Cifrado | 100 MB | ~180ms |
| Apertura completa | 1 MB | ~650ms |
| Apertura completa | 100 MB | ~800ms |

*Hardware: Ryzen 5 5600X, 32 GB RAM. Benchmarks con [Criterion](https://github.com/bheisler/criterion.rs).*

---

## Documentación

- [Threat Model](THREAT_MODEL.md)
- [File Format Specification](FILE_FORMAT.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

---

## Licencia

Apache 2.0 — [LICENSE](LICENSE)

---

**USB Stash** — Tu stash personal portable. Sin instalación, sin servidores, sin compromisos.
