# NullMark Security Policy

## Security objectives

NullMark processes potentially adversarial text, media and document containers. The application should remain safe under malformed Unicode, corrupt structures, hostile archives and excessive metadata.

Primary objectives:

1. No RCE primitive from untrusted text.
2. No HTML/script interpretation of imported text.
3. Minimal renderer-to-native privileges.
4. No network dependency for processing.
5. Bounded resource use.
6. Deterministic and inspectable transformations.
7. No source-file overwrite.

## Renderer hardening

- imported text is assigned to React state and textarea values only
- no `innerHTML` / `dangerouslySetInnerHTML`
- CSP defaults to `none` and explicitly permits only required local resources and Tauri IPC
- no CDN or remote frontend resources
- global Tauri object disabled
- JavaScript prototypes frozen by Tauri configuration

## Native hardening

The Rust crate declares `#![forbid(unsafe_code)]`.

No shell, filesystem, HTTP, opener or process plugin is linked. The main capability contains only four application-specific text/binary permissions.

Commands validate the payload size independently of renderer validation. Do not remove the native limit even if a future UI implements its own limit.

## Resource exhaustion

The current ceilings are 8 MiB UTF-8 text, 32 MiB per binary file, 128 MiB per UI batch, 4,096 ZIP entries, 128 MiB ZIP expansion, 100,000 PDF objects and 128 MiB aggregate PDF stream storage.

Analysis aggregates findings by code point while streaming through the input. It stores at most eight displayed positions per code point, preventing a file consisting of millions of zero-width characters from generating millions of report objects.

## Supply chain

Before release:

- keep a committed lockfile
- run `cargo audit`
- run `npm audit`
- review Tauri/Vite/React release notes before upgrades
- build in CI from a clean checkout
- sign Windows/macOS release artifacts
- generate and publish checksums

`npm run security:check` is a local invariant check, not a substitute for dependency auditing.

## Reporting

Do not include private source documents in vulnerability reports. Reproduce with synthetic text whenever possible.
