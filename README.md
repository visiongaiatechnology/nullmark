# NullMark Beta 1.0

Local-first invisible Unicode watermark and text sanitation desktop application by VisionGaia Technology.

## What this beta does

NullMark Beta 1.0 processes text plus bounded local batches of PNG, JPEG, WebP, PDF, SVG, DOCX, XLSX, PPTX and ODT files. It scans text/XML for invisible Unicode payloads, removes supported container metadata, then independently reparses and re-analyzes every result before export is enabled.

Current detection includes:

- zero-width spaces and joiners
- Unicode tag characters
- word joiners and BOM/zero-width no-break characters
- bidi embeddings, overrides and isolates
- invisible mathematical operators
- variation selectors
- deprecated/invisible format controls
- unusual Unicode spacing characters
- ASCII/C1 control payloads
- private-use Unicode payloads
- token-facing compatibility typography in Maximum mode
- PNG textual/EXIF/time metadata and C2PA `caBX` chunks
- JPEG EXIF/XMP/IPTC/comments and C2PA APP11 fragment sequences
- WebP EXIF/XMP plus RIFF `C2PA` chunks with corrected VP8X flags
- PDF Info, document/page XMP, annotation identity fields, JavaScript/launch actions and C2PA associated-file manifests
- DOCX package properties, revision author/date attributes and ZIP C2PA manifests
- XLSX package properties, comment/person authorship, shared-string Unicode and ZIP C2PA manifests
- PPTX package properties, comment authorship, slide Unicode and ZIP C2PA manifests
- ODT package metadata and ZIP C2PA manifests
- SVG metadata, comments, scripts, event handlers, external references and invisible Unicode

The application does **not** call a cloud service and does not need network access for processing.

## Vendor watermark coverage

Gemini's documented SynthID Text is a probabilistic token-sampling signal, not a hidden Unicode character. NullMark can remove explicit Unicode payloads and Maximum mode can canonicalize token-facing typography, but no local rule engine can prove removal of a proprietary statistical signal without that vendor's configured detector.

OpenAI publicly describes text-watermark research and its vulnerability to broad rewriting, but the cited public material does not establish a universally deployed ChatGPT text watermark. No official public Claude text-watermark specification was found in the Beta 1.0 research pass. NullMark therefore reports deterministic evidence and never converts an unknown vendor state into a false `verified clean` claim. See [docs/WATERMARK_RESEARCH.md](docs/WATERMARK_RESEARCH.md).

## Security model

The beta is deliberately more restricted than a normal Tauri app:

- Tauri 2 + Rust core + React/TypeScript UI
- no shell plugin
- no filesystem plugin
- no HTTP plugin
- no opener/process plugin
- no native file paths sent to Rust; bounded bytes cross IPC
- only four custom IPC commands: text/binary analysis and sanitation
- explicit Tauri permissions for those four commands only
- `withGlobalTauri: false`
- `freezePrototype: true`
- strict production CSP with `default-src 'none'`
- no remote scripts, fonts, images or CDNs
- no `dangerouslySetInnerHTML`
- no `eval` / dynamic function construction
- Rust crate forbids application `unsafe` code
- 8 MiB text and 32 MiB per-file ceilings in both UI and Rust
- ZIP limits: 4,096 entries, 32 MiB/entry and 128 MiB expanded total
- PDF limits: 100,000 objects and 128 MiB aggregate parsed stream storage
- streaming finding aggregation to resist marker-flood memory amplification
- SHA-256 input/output analysis fingerprints
- automatic post-sanitization verification

Run the source security gate at any time:

```bash
npm run security:check
```

## Safe vs Strict

### Safe

Removes markers with a high confidence of being expendable in ordinary text, such as zero-width space, word joiner, soft hyphen, Unicode tag characters and BOM characters.

Context-sensitive characters such as ZWJ/ZWNJ, bidi controls, variation selectors and unusual spaces remain present and are reported.

### Strict

Also removes context-sensitive invisible format characters and normalizes unusual Unicode spaces to ASCII space.

Strict mode can affect rendering or semantics in complex scripts, mathematical notation and emoji sequences. The UI intentionally warns about this.

### Maximum

Also removes private-use payloads and canonicalizes curly quotes, dash variants, ellipsis characters and fullwidth ASCII. This changes the token surface and can weaken some edit-sensitive signatures, but it is not presented as proof that a private statistical watermark has been removed.

## Windows development

Current Vite 8 requires Node.js 20.19+ or 22.12+. Tauri development on Windows also requires Rust, Microsoft C++ Build Tools and WebView2.

From this folder:

```cmd
START_WINDOWS.cmd
```

Or manually:

```bash
npm ci --ignore-scripts
npm run security:check
npm run tauri:dev
```

To build installers:

```cmd
BUILD_WINDOWS.cmd
```

Tauri will place release artifacts under `src-tauri/target/release/bundle/`.

## macOS / Linux

Install the platform prerequisites documented by Tauri, then:

```bash
npm install
npm run security:check
npm run tauri:dev
```

Build with:

```bash
npm run tauri:build
```

## Core test without Tauri

`src-tauri/src/engine.rs` intentionally has no external dependencies. With Rust installed you can validate the sanitizer core independently:

```bash
rustc --edition 2021 --test src-tauri/src/engine.rs -o nullmark-engine-tests
./nullmark-engine-tests
```

On Windows, run `TEST_ENGINE.cmd`.

## Beta limitations

Regular PDF attachments remain preserved; the active-content policy removes JavaScript, launch and automatic actions but does not silently delete unrelated embedded files. Office transformations target package metadata, supported authorship fields and deterministic invisible-Unicode rules, not macros or arbitrary embedded objects.

The product does not claim that every possible AI watermarking method is detectable. It reports only explicitly implemented, auditable rules and returns a separate non-verifiable state for proprietary token-distribution watermarks.

## Delivered in Beta 2

Beta 2 adds an exact split diff and bounded change ledger, XLSX/PPTX/SVG support,
deeper PDF object inspection, German/English interface catalogs and a restrained
desktop UI with a project-owned SVG identity. Verification details and remaining
follow-up work are maintained in [ROADMAP.md](ROADMAP.md).

## Project layout

```text
src/                         React/TypeScript renderer
  App.tsx                    Main UI
  lib/backend.ts             Typed IPC boundary
src-tauri/
  src/engine.rs              Engine boundary and invariants
  src/engine/                Rules, model and sanitizer policies
  src-tauri/src/binary/      PNG/JPEG/WebP/PDF/SVG/Office/ODF parsers
  src/lib.rs                 Bounded Tauri command layer + verification
  permissions/               Explicit custom-command permissions
  capabilities/              Single-window least-privilege capability
  tauri.conf.json            CSP and desktop configuration
scripts/security-check.mjs   Source/config security gate
docs/                        Architecture and threat model
ROADMAP.md                   Versioned future work and acceptance gates
.github/                     CI, Dependabot and PR policy
```
