# Threat Model — USB Stash

> **v1.0** · 2026-05-26 · [Report a vulnerability → SECURITY.md](SECURITY.md)

## At a Glance

| Pregunta | Respuesta |
|----------|-----------|
| ¿Qué protege? | Contenido y nombres de archivos dentro del stash |
| ¿Contra quién? | Ladrón oportunista, atacante con GPU |
| ¿Contra quién NO? | Malware en el host, coerción física, Estado-nación |
| ¿Algoritmo de cifrado? | XChaCha20-Poly1305 (AEAD) |
| ¿Derivación de clave? | Argon2id (64 MB, 3 iteraciones) |
| ¿Se puede romper? | Solo con contraseña débil o malware en el host |

---

## 1. Overview

USB Stash is a portable encrypted vault that lives on a USB drive. This document
describes what the product protects, what it does not, the adversary models we
design against, and the cryptographic guarantees provided.

## 2. What We Protect

| Asset | Protection Mechanism |
|-------|---------------------|
| **File contents** | XChaCha20-Poly1305 AEAD encryption — ciphertext is indistinguishable from random noise without the key |
| **File names and directory structure** | All metadata (paths, sizes, timestamps) is serialized inside the encrypted payload — no plaintext filenames are exposed |
| **Password against offline brute-force** | Argon2id key derivation (64 MB memory, 3 iterations, 4 parallelism) makes GPU/ASIC attacks expensive |
| **Ciphertext tampering** | Poly1305 authentication tag detects any modification — decryption fails if even one byte is altered |
| **Key material in RAM** | `zeroize` trait clears master key and sensitive buffers when the stash is locked or the app exits |
| **Salt and nonce reuse** | 16-byte random salt per stash; 24-byte XChaCha20 nonce eliminates collision risk |

## 3. What We Do NOT Protect

| Threat | Why | Mitigation (if any) |
|--------|-----|---------------------|
| **Keyloggers / screen recorders on the host** | The app runs on an untrusted host; malware can capture keystrokes and screenshots | Advise users to only run on trusted machines |
| **Physical coercion** | No plausible deniability or panic password in v1 | Future: panic password opens a decoy stash |
| **RAM forensics while stash is open** | Decrypted content and master key exist in RAM during use | Zeroize on lock/exit; minimize time stash is open |
| **Camera observing keyboard** | Out of application scope | User awareness |
| **Host filesystem temp files from previews** | The app renders previews in-memory; no temp files created by USB Stash itself | The host OS may still create caches (e.g., font caches) |
| **Side-channel attacks (timing, power)** | Not in scope for v1; software implementations are not constant-time for all operations | Use constant-time comparisons for auth tags (via `subtle` crate) |

## 4. Adversary Models

### 4.1 Opportunistic Thief (Low Capability)
- **Access:** Physical possession of the USB drive.
- **Skills:** Basic file browsing, no cryptographic knowledge.
- **Mitigation:** Fully mitigated. The `stash.dat` file is encrypted; `stash.meta` contains only public parameters (salt, algorithm names, version).

### 4.2 Deterministic Attacker (Medium Capability)
- **Access:** Physical possession of the USB drive + dedicated GPU cluster for brute-force.
- **Skills:** Can run hashcat/john, understands crypto basics.
- **Mitigation:** Argon2id with 64 MB memory makes each guess cost ~64 MB of GPU RAM. At 3 iterations, a single guess takes ~100-500ms on consumer hardware. A 12+ character password with medium entropy provides sufficient resistance.

### 4.3 Advanced Attacker (High Capability)
- **Access:** Physical possession + custom hardware + knowledge of the file format.
- **Skills:** Can implement custom attacks against the binary format, may attempt differential cryptanalysis.
- **Mitigation:** XChaCha20-Poly1305 is a well-studied AEAD construction. The 256-bit key space is infeasible to brute-force. The file format is open — security through obscurity is not relied upon.

### 4.4 Nation-State Attacker (Not Considered)
- **Capabilities:** Custom ASICs, zero-day exploits, persistent host compromise.
- **Scope:** Explicitly out of scope for v1. USB Stash is not designed for this threat level.

## 5. Cryptographic Guarantees

| Property | Guarantee | Basis |
|----------|-----------|-------|
| **Confidentiality** | Ciphertext reveals no information about plaintext without the key | XChaCha20 stream cipher (256-bit key) |
| **Integrity** | Any modification to ciphertext is detected | Poly1305 MAC (128-bit tag) |
| **Authenticity** | Only someone with the correct key can produce valid ciphertext | AEAD construction (encrypt-then-MAC) |
| **Key strength** | Derived key is 256 bits, suitable for XChaCha20 | Argon2id output = 32 bytes |
| **Nonce safety** | No nonce reuse across chunks within a stash | 24-byte nonce space (2^192 per chunk) |
| **Password hardness** | Each offline guess costs 64 MB memory + 3 iterations | Argon2id parameters per OWASP 2024 |

## 6. Attack Surface Analysis

### 6.1 Entry Points

| Entry Point | Risk | Mitigation |
|-------------|------|------------|
| **Password input** | Keylogging, shoulder surfing | App provides no mitigation; user must ensure trusted environment |
| **stash.dat file** | Tampering, truncation, replacement | AEAD tag verification fails on any modification |
| **stash.meta file** | Tampering of KDF parameters | If parameters are changed, key derivation produces wrong key → decryption fails |
| **CLI arguments** | Password exposed in process list | Password read from stdin/prompt, not CLI args |
| **GUI input fields** | Keylogging, shoulder surfing | App provides no mitigation; user must ensure trusted environment |
| **File system (host)** | Host reads/modifies stash files while app is closed | Encryption protects at rest; app detects tampering on open |

### 6.2 Data Flow

```
User Password → Argon2id(salt) → 256-bit Master Key
Master Key + Nonce → XChaCha20-Poly1305 → Ciphertext + Tag
Ciphertext + Tag → stash.dat (on disk)
```

At no point is the password, master key, or plaintext written to disk.

### 6.3 Memory Safety

- Rust's ownership model prevents use-after-free and double-free vulnerabilities.
- `zeroize` ensures sensitive data is cleared from memory when no longer needed.
- No `unsafe` blocks in the cryptographic code path (verified by `cargo clippy`).

## 7. Assumptions

1. The user chooses a password with sufficient entropy (≥ 12 characters, mixed character types).
2. The USB drive is not physically modified (e.g., firmware-level attack).
3. The host OS is not compromised at the kernel level.
4. The Rust standard library and cryptographic crates are free of vulnerabilities.

## 8. Version History

| Date | Version | Changes |
|------|---------|---------|
| 2026-05-26 | 1.0 | Initial threat model |
