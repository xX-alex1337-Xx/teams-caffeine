@echo off
setlocal
cd /d "%~dp0"

if exist "target\release\teams-caffeine.exe" (
    start "" "target\release\teams-caffeine.exe"
    exit /b 0
)

if exist "teams-caffeine.exe" (
    start "" "teams-caffeine.exe"
    exit /b 0
)

echo Building and starting Teams Caffeine...
cargo run --release
