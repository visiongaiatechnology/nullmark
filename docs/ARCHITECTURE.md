# NullMark Beta 1.0 Architecture

## Trust boundary

```text
Untrusted text/file bytes
        |
        v
+-----------------------+
| React WebView         |
| - textarea only       |
| - no HTML execution   |
| - 8 MiB guard         |
+-----------+-----------+
            |
            | typed IPC: text + mode only
            v
+-----------------------+
| Tauri command gate    |
| - 8 MiB guard again   |
| - only 2 commands     |
+-----------+-----------+
            |
            v
+-----------------------+
| Rust sanitizer engine |
| - deterministic rules |
| - no I/O              |
| - no network          |
| - no unsafe code      |
+-----------+-----------+
            |
            v
    analysis / output
            |
            v
+-----------------------+
| Independent re-scan   |
| verification result   |
+-----------------------+
```

The renderer is treated as a lower-trust component. A renderer compromise should not grant an attacker a generic primitive to execute commands, read arbitrary files or connect to remote hosts through Tauri plugins.

## Why files are not native in 0.1

The browser/WebView File API reads a user-dropped or user-selected text file. Only decoded text is passed to Rust. This avoids granting filesystem capability to the renderer during the first beta.

A later document/image module should use a dedicated native ingestion API with scoped file handles rather than adding unrestricted filesystem access.

## Command surface

Only:

- `analyze_text(text)`
- `sanitize_text(text, mode)`

There is intentionally no generic command dispatcher, no file command, no URL command and no shell command.

## Verification

Sanitization is followed by a fresh analysis pass. `verification_passed` means no character actionable in the selected mode remains.

`canonical_projection_unchanged` is a deterministic sanitizer invariant, **not** a universal promise of identical typography or meaning. ZWJ, ZWNJ, variation selectors, bidi controls, soft hyphens and word joiners can affect rendering. Strict and Maximum modes are explicitly semantic-review operations.

The response exposes `verification_scope` and keeps proprietary probabilistic token watermarks in `not-verifiable-without-vendor-detector` state. This prevents the UI from presenting a deterministic Unicode rescan as universal vendor-watermark proof.
