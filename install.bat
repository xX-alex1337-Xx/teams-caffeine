@echo off
setlocal
cd /d "%~dp0"

echo ===================================================
echo Teams Caffeine - Windows Installer
echo ===================================================

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Installation encountered an error.
    pause
    exit /b %ERRORLEVEL%
)

pause
