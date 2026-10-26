# =======================================================
# Teams Caffeine - Uninstaller Script
# =======================================================

$ErrorActionPreference = "SilentlyContinue"

Write-Host "`n[1/4] Stopping Teams Caffeine..." -ForegroundColor Cyan
Get-Process -Name 'teams-caffeine' | Stop-Process -Force
Start-Sleep -Milliseconds 600

Write-Host "[2/4] Removing Windows Autostart..." -ForegroundColor Cyan
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "TeamsCaffeine" -ErrorAction SilentlyContinue

Write-Host "[3/4] Removing Start Menu Shortcut..." -ForegroundColor Cyan
$ShortcutPath = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Teams Caffeine.lnk"
if (Test-Path $ShortcutPath) {
    Remove-Item $ShortcutPath -Force
}

Write-Host "[4/4] Removing Program Files..." -ForegroundColor Cyan
$InstallDir = "$env:LOCALAPPDATA\Programs\TeamsCaffeine"
if (Test-Path $InstallDir) {
    Remove-Item $InstallDir -Recurse -Force
}

Write-Host "`nTeams Caffeine has been successfully uninstalled.`n" -ForegroundColor Green
