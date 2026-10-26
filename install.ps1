# =======================================================
# Teams Caffeine - Installer Script
# Installs Windows System Tray App without admin rights
# =======================================================

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $ScriptDir) { $ScriptDir = Get-Location }

$InstallDir = "$env:LOCALAPPDATA\Programs\TeamsCaffeine"
$StartMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$ShortcutPath = Join-Path $StartMenuDir "Teams Caffeine.lnk"
$TargetExe = Join-Path $InstallDir "teams-caffeine.exe"

Write-Host "`n[1/5] Checking for binary..." -ForegroundColor Cyan
$SourceExe = Join-Path $ScriptDir "target\release\teams-caffeine.exe"
if (-not (Test-Path $SourceExe)) {
    $SourceExe = Join-Path $ScriptDir "teams-caffeine.exe"
}

if (-not (Test-Path $SourceExe)) {
    Write-Host "No precompiled .exe found. Building release with Cargo..." -ForegroundColor Yellow
    & cargo build --release
    $SourceExe = Join-Path $ScriptDir "target\release\teams-caffeine.exe"
    if (-not (Test-Path $SourceExe)) {
        Write-Error "Build failed or 'teams-caffeine.exe' not found."
        exit 1
    }
}

Write-Host "[2/5] Stopping any running instances..." -ForegroundColor Cyan
Get-Process -Name 'teams-caffeine' -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 600

Write-Host "[3/5] Installing application files to $InstallDir..." -ForegroundColor Cyan
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}
Copy-Item -Path $SourceExe -Destination $TargetExe -Force

# Copy uninstall scripts to program directory
if (Test-Path (Join-Path $ScriptDir "uninstall.bat")) {
    Copy-Item (Join-Path $ScriptDir "uninstall.bat") (Join-Path $InstallDir "uninstall.bat") -Force -ErrorAction SilentlyContinue
}
if (Test-Path (Join-Path $ScriptDir "uninstall.ps1")) {
    Copy-Item (Join-Path $ScriptDir "uninstall.ps1") (Join-Path $InstallDir "uninstall.ps1") -Force -ErrorAction SilentlyContinue
}

Write-Host "[4/5] Creating Start Menu shortcut..." -ForegroundColor Cyan
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut($ShortcutPath)
$Shortcut.TargetPath = $TargetExe
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Description = "Teams Caffeine - Keeps PC awake and MS Teams active"
$Shortcut.IconLocation = "$TargetExe,0"
$Shortcut.Save()

Write-Host "[5/5] Launching Teams Caffeine..." -ForegroundColor Green
Start-Process -FilePath $TargetExe

Write-Host "`nInstallation successful!" -ForegroundColor Green
Write-Host "- Program installed at: $TargetExe"
Write-Host "- Start Menu shortcut:  $ShortcutPath"
Write-Host "- Look for the coffee icon in your Windows System Tray (bottom right notification area).`n"
