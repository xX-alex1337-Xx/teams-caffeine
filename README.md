# ☕ Teams Caffeine

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://microsoft.com)
[![Binary Size](https://img.shields.io/badge/binary-~280_KB-informational.svg)]()

> Lightweight, native Windows system tray utility written in pure Rust that keeps your computer awake and MS Teams status persistently **Active (Green)** — with virtually zero memory and CPU overhead.

<p align="center">
  <img src="assets/preview.png" alt="Teams Caffeine Icon" width="140" />
</p>

---

## 🎯 Purpose & Background

Many corporate Windows machines lock automatically or put Microsoft Teams / Slack / Skype into "Away / Idle" (Yellow) status after a few minutes of inactivity. 

Common PowerShell workarounds like:
```powershell
$wscript = New-Object -ComObject Wscript.Shell
while ($true) {
    $wscript.SendKeys('{F15}')
    Start-Sleep -Seconds 30
}
```
leave an open PowerShell window, consume ~50–80 MB of RAM, and can be terminated by script policies or accidental window closures.

**Teams Caffeine** replaces this with a native, tiny (~280 KB) Windows background utility that lives unobtrusively in your system tray:
1. Emulates `{F15}` keypresses via native Win32 `SendInput` every 30 seconds (F15 is completely harmless and does not affect normal typing or active applications).
2. Performs a subtle **1-pixel relative mouse movement** forth and back (`+1px`, then `-1px`) every 30 seconds for environments that monitor mouse telemetry. Net displacement is zero, with zero cursor drift.
3. Asserts Windows Power Execution State (`SetThreadExecutionState`) to prevent screen lock, screensavers, or sleep mode while active.
4. Consumes **< 2 MB RAM** and virtually **0% CPU**.

---

## ✨ Features

- **🚀 Ultra Lightweight**: Standalone ~280 KB native executable, no heavy frameworks, no Electron, no WebView.
- **☕ Visual Tray Feedback**:
  - **Active State**: Colorful coffee mug with clean white steam and a solid green indicator badge.
  - **Paused State**: Dimmed grayscale coffee mug with pause icon.
- **⚡ Instant Toggle**: Left-click or double-click the tray icon to quickly toggle between **Active** and **Paused**.
- **📋 Context Menu (Right-Click)**:
  - Current status display
  - Active / Paused toggle checkmark
  - **Start with Windows** autostart toggle (saves cleanly to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
  - Exit
- **🔒 Single Instance Protection**: Prevents duplicate instances using a named Windows Mutex.
- **📦 No Admin Rights Required**: Can be installed and run by standard Windows user accounts.

---

## 🛠️ Quick Installation

### Option 1: One-Click Installer (Recommended)
Double-click `install.bat` (or run in PowerShell):
```powershell
.\install.ps1
```
This will:
- Copy `teams-caffeine.exe` to `%LOCALAPPDATA%\Programs\TeamsCaffeine`
- Add a shortcut in your Windows Start Menu
- Launch Teams Caffeine in the system tray

### Option 2: Portable / Manual
Download or build `teams-caffeine.exe` and place it anywhere you like. Simply double-click to launch!

---

## ⌨️ How It Works Under the Hood

### 1. Teams Activity Emulation (`VK_F15` & 1px Mouse Movement)
Microsoft Teams monitors Windows user idle time (`GetLastInputInfo`). Teams Caffeine uses a dual-action trigger every 30 seconds:
- Simulates an invisible, non-printable key like **F15** (`0x7E`).
- Jiggles the mouse cursor by **1 pixel right and 1 pixel left** using relative offsets (`MOUSEEVENTF_MOVE`).

```rust
// 1) Simulate F15 keypress
let key_inputs = [
    INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_F15, .. } } },
    INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_F15, dwFlags: KEYEVENTF_KEYUP, .. } } },
];
SendInput(&key_inputs, ...);

// 2) Jiggle mouse 1px forth and back (net displacement 0)
let mouse_inputs = [
    INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dx: 1, dy: 0, dwFlags: MOUSEEVENTF_MOVE, .. } } },
    INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dx: -1, dy: 0, dwFlags: MOUSEEVENTF_MOVE, .. } } },
];
SendInput(&mouse_inputs, ...);
```

### 2. Display & System Sleep Prevention
While in **Active** mode, Teams Caffeine notifies the Windows Power Manager via:
```rust
SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
```
When switched to **Paused** mode or upon exit, normal power saving is immediately restored:
```rust
SetThreadExecutionState(ES_CONTINUOUS);
```

---

## 🏗️ Building from Source

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (stable channel, `x86_64-pc-windows-msvc` or `x86_64-pc-windows-gnu`)
- Windows 10 / 11

### Build Release Binary
```bash
cargo build --release
```
The optimized `.exe` will be located at `target/release/teams-caffeine.exe`.

---

## 🗑️ Uninstallation

Run `uninstall.bat` or in PowerShell:
```powershell
.\uninstall.ps1
```
This safely stops the application, removes the Windows autostart registry entry, and removes installed files and shortcuts.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
