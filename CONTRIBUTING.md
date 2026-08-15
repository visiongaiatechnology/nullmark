# Contributing to NullMark

NullMark accepts narrowly scoped, auditable changes. Every transformation must
be deterministic, covered by tests, and classified by semantic risk.

## Required checks

```powershell
npm ci
npm test
rustc --edition 2021 --test src-tauri/src/engine.rs -o .tmp/nullmark-engine-tests.exe
.tmp/nullmark-engine-tests.exe
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm audit --audit-level=high
```

Never add remote renderer content, generic shell/filesystem commands, telemetry,
or an unbounded parser. Security-sensitive changes require a threat-model update.

## Pull requests

- Explain the transformation and its semantic risk.
- Add positive, negative, idempotency, and adversarial tests.
- Do not claim vendor watermark removal without a vendor detector and reproducible evidence.
- Keep `package-lock.json` and `src-tauri/Cargo.lock` committed.
