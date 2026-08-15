@echo off
setlocal
cd /d "%~dp0"
where rustc >nul 2>nul || (echo ERROR: rustc missing.& exit /b 1)
if not exist .tmp mkdir .tmp
rustc --edition 2021 -D warnings --test src-tauri\src\engine.rs -o .tmp\nullmark-engine-tests.exe
if errorlevel 1 exit /b 1
.tmp\nullmark-engine-tests.exe
set ERR=%ERRORLEVEL%
del /q .tmp\nullmark-engine-tests.exe >nul 2>nul
exit /b %ERR%
