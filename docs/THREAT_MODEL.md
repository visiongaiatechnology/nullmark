# Threat model

## Assets and trust boundaries

Untrusted text or bounded file bytes enter a lower-trust React renderer and cross
one typed Tauri IPC boundary. Rust revalidates byte limits before deterministic,
network-free processing. The renderer receives data, never executable markup.

Protected assets are the host account, local files, clipboard contents, source
documents, output integrity, and the accuracy of the verification claim.

## Defenses

- No remote renderer resources, CDN, telemetry, HTTP, shell, process, opener, or filesystem plugin.
- Production CSP starts at `default-src 'none'`; no `unsafe-inline` or `unsafe-eval`.
- `withGlobalTauri: false`, frozen prototypes, and only four custom command permissions.
- No `innerHTML`, `dangerouslySetInnerHTML`, dynamic function construction, or network primitive.
- Native 8 MiB text and 32 MiB file ceilings independent of renderer validation.
- Strict magic-byte parsing, ZIP path/overlap/encryption/compression limits, PDF object/stream limits, and format-specific structural checks.
- Streaming aggregation with at most eight recorded positions per code point.
- Rust application code forbids `unsafe`.
- Input revision binding prevents stale analysis from being attached to edited text.
- Sanitization is followed by a fresh parser/classifier pass; export remains blocked unless supported findings reach zero.
- SHA-256 fingerprints identify the exact before/after payloads.
- Lockfiles, CI builds, and high-severity dependency audits are mandatory.

## Explicit non-goals

- Executing or rendering imported HTML.
- Opening arbitrary native paths.
- Claiming that absence of known code points proves human authorship.
- Claiming removal of a proprietary probabilistic signal without its detector.
- Executing macros, scripts, active PDF content, or embedded office objects.
- Claiming PDF metadata coverage beyond Info and catalog XMP.
- Validating C2PA signatures or provenance claims; NullMark detects and removes the specified embedded carriers but is not a trust validator.

Files remain browser-provided byte arrays: no generic native filesystem capability,
path authority or source overwrite exists. ZIP containers are rebuilt in memory
with safe names and deterministic timestamps; every cleaned format is reopened and reparsed.
