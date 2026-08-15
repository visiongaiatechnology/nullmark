@echo off
setlocal
cd /d "%~dp0"

echo [NullMark] Development bootstrap

where node >nul 2>nul || (
  echo ERROR: Node.js is not installed or not in PATH.
  exit /b 1
)
where npm >nul 2>nul || (
  echo ERROR: npm is not installed or not in PATH.
  exit /b 1
)
where cargo >nul 2>nul || (
  echo ERROR: Rust/Cargo is not installed or not in PATH.
  echo Install Rust using the official rustup installer, then reopen this terminal.
  exit /b 1
)

node -e "const [M,m]=process.versions.node.split('.').map(Number); if(!((M===20&&m>=19)||(M===22&&m>=12)||M>22)){console.error('ERROR: Vite 8 requires Node 20.19+ or 22.12+. Current:',process.versions.node);process.exit(1)}"
if errorlevel 1 exit /b 1

if not exist node_modules (
  echo [NullMark] Installing JavaScript dependencies...
  call npm ci --ignore-scripts
  if errorlevel 1 exit /b 1
)

call npm run security:check
if errorlevel 1 exit /b 1

echo [NullMark] Starting Tauri development app...
call npm run tauri:dev
