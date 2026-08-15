# Beta release procedure

## Reproducible Windows build

Prerequisites: Node.js 22 LTS, current stable Rust MSVC toolchain, Microsoft C++
Build Tools, WebView2, WiX and NSIS support required by Tauri.

Run from a clean checkout:

```powershell
BUILD_WINDOWS.cmd
```

The gate performs `npm ci --ignore-scripts`, npm high-severity audit, source and
configuration security checks, standalone engine tests, locked Cargo tests, the
production frontend build, and native MSI/NSIS packaging.

Artifacts are written below `src-tauri/target/release/bundle/`. Generate SHA-256
checksums from the final files, test both installer and uninstaller in a disposable
Windows VM, then sign the installers and executable with the organization EV/OV
code-signing certificate. Unsigned test artifacts must be labeled `UNSIGNED`.

## GitHub release gate

- CI green from a clean checkout.
- `npm audit --audit-level=high` and `cargo audit` have no unaccepted findings.
- Version matches in package.json, Cargo.toml and tauri.conf.json.
- Changelog and watermark research baseline are current.
- Installer smoke-tested with network disconnected.
- Analyze, Safe, Strict, Maximum, copy and export flows exercised.
- Release notes state the deterministic verification scope and statistical-watermark limitation.

Never commit signing keys, certificate passwords, timestamp-service credentials,
or private vendor watermark detector configurations.
