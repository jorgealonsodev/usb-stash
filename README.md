# USB Stash

> **Tu bóveda cifrada portable. Metela en un USB, ejecutala en cualquier PC. Cero instalación.**

[![Version](https://img.shields.io/badge/version-0.1.0-blue)](CHANGELOG.md)
[![License](https://img.shields.io/badge/license-Apache%202.0-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://rust-lang.org)

USB Stash cifra tus archivos con **XChaCha20-Poly1305** y los guarda en un único
contenedor portable. Lo llevás en un USB. Lo abrís con doble click. Sin instalar
nada. Sin conexión a internet. Sin servidores. Solo vos y tu contraseña.

---

## Quick path

```bash
# 1. Copiá el binario a tu USB (Windows o Linux)
# 2. Ejecutalo con doble click
# 3. Creá tu stash con una contraseña fuerte (≥ 12 caracteres)
# 4. Arrastrá archivos, previsualizá, exportá
```

---

## ¿Qué hace?

| Hace | No hace |
|------|---------|
| Cifra archivos con XChaCha20-Poly1305 (AEAD) | No se conecta a internet |
| Protege la contraseña con Argon2id (64 MB) | No crea archivos temporales |
| Detecta cualquier modificación del archivo | No requiere instalación |
| Funciona en Windows 10+ y Linux | No tiene servidores ni cuentas |
| Zeroiza claves y datos al cerrar | No recupera contraseñas perdidas |

---

## Stack

| Capa | Tecnología | Por qué |
|------|------------|---------|
| Cripto | `argon2` + `chacha20poly1305` | Moderno, auditado, sin AES-NI requerido |
| Backend | Rust + Tauri 2 | Binarios chicos, memory safety, cross-platform |
| Frontend | Svelte 4 + TypeScript | Bundle mínimo, reactividad nativa |
| Build | Cargo workspace + Vite | Monorepo, un solo `cargo build` |

```
┌──────────────────────────────────────────┐
│              USB DRIVE                   │
│  usbstash-win.exe / usbstash-linux       │
│  stash.dat  (bóveda cifrada)             │
│  stash.meta (parámetros públicos)        │
└──────────────┬───────────────────────────┘
               ▼
┌──────────────────────────────────────────┐
│         TAURI APP (USB Stash)            │
│  ┌─ Frontend Svelte ──────────────────┐  │
│  │  Login · Create · Explorer · Preview│  │
│  │  Settings · Auto-lock · Export     │  │
│  └──────────┬──────────────────────────┘  │
│             │ Tauri IPC                   │
│  ┌──────────▼──────────────────────────┐  │
│  │  Backend Rust (usbstash-core)       │  │
│  │  Argon2id · AEAD · Format · Stash   │  │
│  └─────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

---

## Uso

### Usuario final

| SO | Instrucción |
|----|-------------|
| **Windows** | Doble click en `usbstash-win.exe` |
| **Linux** | `chmod +x usbstash-linux && ./usbstash-linux` |

### CLI (desarrolladores)

```bash
usbstash create /ruta/stash          # Crear bóveda nueva
usbstash add /ruta/stash doc.pdf     # Agregar archivo
usbstash list /ruta/stash            # Listar contenido
usbstash extract /ruta/stash doc.pdf # Extraer archivo
```

---

## Desarrollo

### Prerrequisitos

| Dependencia | Versión | Instalación |
|-------------|---------|-------------|
| Rust | 1.75+ | `rustup install stable` |
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |
| pnpm | — | `npm install -g pnpm` |
| webkit2gtk (Linux) | 4.1-dev | `sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev` |
| WebView2 (Windows) | — | Incluido en Win 10+ |

### Comandos

```bash
cargo build                    # Backend
cd frontend && pnpm install && pnpm build  # Frontend
cargo tauri build              # App completa
./scripts/build-portable.sh    # Distribución USB (Linux)
```

### Tests y calidad

```bash
cargo test --all                         # 155 tests Rust
cd frontend && pnpm test                 # 41 tests Svelte
cargo clippy --all -- -D warnings        # Lint
cargo fmt --check                        # Formato
```

---

## Rendimiento

| Operación | Tiempo | Hardware |
|-----------|--------|----------|
| Derivar clave (Argon2id) | ~600ms | Ryzen 5 5600X |
| Abrir stash de 1 MB | ~650ms | 32 GB RAM |
| Abrir stash de 100 MB | ~800ms | NVMe SSD |
| Cifrar 100 MB | ~180ms | — |

*Benchmarks con [Criterion](https://github.com/bheisler/criterion.rs).*

---

## Documentación

| Documento | ¿Para quién? |
|-----------|---------------|
| [FILE_FORMAT.md](FILE_FORMAT.md) | Desarrolladores que quieran implementar un decodificador independiente |
| [THREAT_MODEL.md](THREAT_MODEL.md) | Auditores de seguridad, pentesters |
| [SECURITY.md](SECURITY.md) | Quien encuentre una vulnerabilidad |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Quien quiera contribuir código |
| [CHANGELOG.md](CHANGELOG.md) | Historial de versiones |

---

## Decisiones de diseño

| Decisión | Por qué |
|----------|---------|
| XChaCha20 sobre AES-GCM | Nonces de 24 bytes (sin riesgo de colisión), sin requerir AES-NI |
| Argon2id con 64 MB | OWASP 2024: equilibrio entre seguridad y usabilidad interactiva |
| Contenedor único (`stash.dat`) | No se exponen nombres de archivo individuales |
| Cero conexiones de red | Defensa en profundidad: la app no puede leakear datos |
| Svelte sobre React | Bundle más chico, menos abstracciones |
| Bincode sobre JSON | Binario, compacto, determinístico |

---

## Licencia

Apache 2.0 — [LICENSE](LICENSE)
