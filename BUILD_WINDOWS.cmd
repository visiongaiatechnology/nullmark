@echo off
setlocal
cd /d "%~dp0"

echo [NullMark] Release build gate
where node >nul 2>nul || (echo ERROR: Node.js missing.& exit /b 1)
where npm >nul 2>nul || (echo ERROR: npm missing.& exit /b 1)
where cargo >nul 2>nul || (echo ERROR: Rust/Cargo missing.& exit /b 1)

node -e "const [M,m]=process.versions.node.split('.').map(Number); if(!((M===20&&m>=19)||(M===22&&m>=12)||M>22)){console.error('ERROR: Vite 8 requires Node 20.19+ or 22.12+. Current:',process.versions.node);process.exit(1)}"
if errorlevel 1 exit /b 1

call npm ci --ignore-scripts
if errorlevel 1 exit /b 1

call npm audit --audit-level=high
if errorlevel 1 exit /b 1

call npm run security:check
if errorlevel 1 exit /b 1

call TEST_ENGINE.cmd
if errorlevel 1 exit /b 1

cargo test --locked --manifest-path src-tauri\Cargo.toml
if errorlevel 1 exit /b 1

cargo clippy --locked --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings
if errorlevel 1 exit /b 1

where cargo-audit >nul 2>nul || (echo ERROR: cargo-audit missing. Install with cargo install cargo-audit --locked.& exit /b 1)
cargo audit --file src-tauri\Cargo.lock
if errorlevel 1 exit /b 1

call npm run tauri:build -- --bundles nsis,msi
if errorlevel 1 exit /b 1

echo.
echo [NullMark] Build complete. Check src-tauri\target\release\bundle\
