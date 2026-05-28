# Security Policy — USB Stash

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
| < 0.1   | No        |

Only the latest minor release within the current major version receives security
updates. Users are encouraged to upgrade promptly.

## Reporting a Vulnerability

We take the security of USB Stash seriously. If you discover a security
vulnerability, please follow responsible disclosure:

1. **Do NOT open a public issue.** Security issues must be reported privately.
2. **Email:** [security@usbstash.app](mailto:security@usbstash.app)
3. **Include:**
   - A description of the vulnerability
   - Steps to reproduce (proof of concept if possible)
   - Potential impact assessment
   - Your contact information for follow-up

## What to Expect

- **Acknowledgment:** We will acknowledge receipt of your report within **48 hours**.
- **Initial Assessment:** Within **7 days**, we will provide an initial assessment and timeline.
- **Resolution:** We aim to resolve critical vulnerabilities within **30 days**.
- **Disclosure:** We will coordinate public disclosure with you. We prefer a
  coordinated release where the fix is available before the vulnerability is
  publicly disclosed.

## PGP Key

For encrypted communication, our PGP public key will be available at:

```
https://usbstash.app/.well-known/security.asc
```

Until the key is published, please send reports unencrypted to
security@usbstash.app. We will respond promptly.

## Scope

This security policy covers:

- `usbstash-core` — cryptographic operations, file format, stash lifecycle
- `usbstash-cli` — command-line interface
- `usbstash-gui` — egui desktop application

## Out of Scope

- Vulnerabilities in third-party dependencies (report to the respective projects)
- Social engineering attacks against users
- Physical attacks on the USB drive hardware

## Security Best Practices for Users

1. **Use a strong password:** Minimum 12 characters with mixed types.
2. **Keep the app updated:** Security patches are released promptly.
3. **Verify downloads:** Check SHA256 checksums against published releases.
4. **Use on trusted machines:** The app cannot protect against host-level malware.
5. **Lock the stash:** Use the lock feature when stepping away from the computer.

## Acknowledgments

We appreciate security researchers who responsibly disclose vulnerabilities.
Contributors will be acknowledged in our security advisories (with permission).
