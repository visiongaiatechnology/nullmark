# Build status — NullMark 1.0.0-beta.2

Date: 2026-08-15

## Verified

- exact npm direct versions and committed package lock
- Cargo lock and locked native build
- source/config security gate: 28 source/config files
- npm audit: 0 known vulnerabilities
- TypeScript strict compilation and Vite production build
- 22 Rust engine, image, PDF, SVG and Office-container tests
- Rust formatting and Clippy with warnings denied
- RustSec: 0 vulnerabilities; 17 allowed transitive maintenance/unsoundness warnings
- optimized Windows x64 release executable
- NSIS x64 installer
- MSI x64 en-US installer with numeric WiX version 1.0.0.2
- native smoke test: responsive main window NullMark Beta 1.0.0-beta.2
- native UI capture: 1936 × 1048
- SHA-256 checksums recorded in SHA256SUMS.txt
- PNG/JPEG/WebP metadata and C2PA sanitation
- DOCX/XLSX/PPTX/ODT bounded package sanitation and reopen verification
- SVG metadata, active-content, external-reference and Unicode sanitation
- PDF Info/page-XMP/annotation/action/C2PA sanitation with reopen verification
- German and English interface catalogs
- exact bounded text change ledger and split diff

## Release boundary

The generated artifacts are unsigned local test builds. Public distribution
still requires organization code signing, a clean GitHub Actions run and an
installer/uninstaller test in a disposable Windows VM.

Regular PDF attachments are preserved by policy. Proprietary statistical token
watermarks remain non-verifiable without a compatible vendor detector.
