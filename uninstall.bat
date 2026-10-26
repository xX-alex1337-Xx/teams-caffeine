@echo off
setlocal
cd /d "%~dp0"

echo ===================================================
echo Teams Caffeine - Windows Uninstaller
echo ===================================================

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Uninstallation encountered an issue.
    pause
    exit /b %ERRORLEVEL%
)

pause
