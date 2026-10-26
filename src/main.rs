#![windows_subsystem = "windows"]

use std::sync::{Mutex, OnceLock};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ, REG_VALUE_TYPE,
};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VK_F15,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    DispatchMessageW, GetCursorPos, GetMessageW, LoadIconW, PostMessageW, PostQuitMessage,
    RegisterClassW, RegisterWindowMessageW, SetForegroundWindow, SetTimer, TrackPopupMenuEx,
    TranslateMessage, CS_HREDRAW, CS_VREDRAW, HICON, HMENU, MF_CHECKED, MF_DISABLED, MF_GRAYED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RIGHTBUTTON,
    WINDOW_EX_STYLE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_LBUTTONDBLCLK, WM_LBUTTONUP,
    WM_NULL, WM_RBUTTONUP, WM_TIMER, WM_USER, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

const WM_TRAYICON: u32 = WM_USER + 100;
const TIMER_KEEP_AWAKE: usize = 1001;
const KEEP_AWAKE_INTERVAL_MS: u32 = 30_000; // 30 seconds

const ID_TRAY_TITLE: usize = 2000;
const ID_TRAY_TOGGLE: usize = 2001;
const ID_TRAY_AUTOSTART: usize = 2002;
const ID_TRAY_EXIT: usize = 2003;

const REG_VALUE_NAME: &windows::core::HSTRING = windows::core::h!("TeamsCaffeine");

struct AppState {
    active: bool,
    h_icon_active: HICON,
    h_icon_paused: HICON,
    nid: NOTIFYICONDATAW,
}

unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();
static WM_TASKBARCREATED: OnceLock<u32> = OnceLock::new();

fn send_f15_key() {
    let inputs = [
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_F15,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_F15,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];

    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn apply_execution_state(active: bool) {
    unsafe {
        if active {
            // Prevent display sleep, system sleep, and lock
            let _ = SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED);
        } else {
            // Reset to normal power-saving behavior
            let _ = SetThreadExecutionState(ES_CONTINUOUS);
        }
    }
}

fn is_autostart_registered() -> bool {
    unsafe {
        let mut key = HKEY::default();
        let subkey = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_READ, &mut key).is_ok() {
            let mut data_type = REG_VALUE_TYPE::default();
            let mut data_len = 0u32;
            let status = RegQueryValueExW(
                key,
                PCWSTR(REG_VALUE_NAME.as_ptr()),
                None,
                Some(&mut data_type),
                None,
                Some(&mut data_len),
            );
            let _ = RegCloseKey(key);
            status.is_ok()
        } else {
            false
        }
    }
}

fn set_autostart_registered(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let mut key = HKEY::default();
        let subkey = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");
        RegOpenKeyExW(HKEY_CURRENT_USER, subkey, 0, KEY_SET_VALUE, &mut key).ok()?;

        if enable {
            let current_exe = std::env::current_exe()?;
            let exe_str = current_exe.to_str().ok_or("Invalid executable path")?;
            let formatted_path = format!("\"{}\"", exe_str);
            let mut wide_chars: Vec<u16> = formatted_path.encode_utf16().collect();
            wide_chars.push(0);

            let slice = std::slice::from_raw_parts(
                wide_chars.as_ptr() as *const u8,
                wide_chars.len() * 2,
            );

            RegSetValueExW(
                key,
                PCWSTR(REG_VALUE_NAME.as_ptr()),
                0,
                REG_SZ,
                Some(slice),
            ).ok()?;
        } else {
            let _ = RegDeleteValueW(key, PCWSTR(REG_VALUE_NAME.as_ptr()));
        }

        let _ = RegCloseKey(key);
    }
    Ok(())
}

fn set_tooltip(nid: &mut NOTIFYICONDATAW, tip: &str) {
    let mut tip_wide = [0u16; 128];
    let encoded: Vec<u16> = tip.encode_utf16().collect();
    let len = encoded.len().min(127);
    tip_wide[..len].copy_from_slice(&encoded[..len]);
    nid.szTip = tip_wide;
}

fn update_tray_visuals(active: bool) {
    if let Some(state_lock) = STATE.get() {
        if let Ok(mut state) = state_lock.lock() {
            state.active = active;
            state.nid.hIcon = if active {
                state.h_icon_active
            } else {
                state.h_icon_paused
            };

            let tip = if active {
                "Teams Caffeine: Active (F15 every 30s)"
            } else {
                "Teams Caffeine: Paused"
            };
            set_tooltip(&mut state.nid, tip);

            unsafe {
                let _ = Shell_NotifyIconW(NIM_MODIFY, &state.nid);
            }
        }
    }
}

fn toggle_active(hwnd: HWND) {
    let current_state = if let Some(state_lock) = STATE.get() {
        if let Ok(state) = state_lock.lock() {
            state.active
        } else {
            true
        }
    } else {
        true
    };

    let new_state = !current_state;
    apply_execution_state(new_state);
    if new_state {
        send_f15_key();
        unsafe {
            // Reset timer so the next interval starts fresh 30 seconds from now
            SetTimer(hwnd, TIMER_KEEP_AWAKE, KEEP_AWAKE_INTERVAL_MS, None);
        }
    }
    update_tray_visuals(new_state);
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hinstance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);

            // Load active (ID 1) and paused (ID 2) icons from resources
            let h_icon_active = LoadIconW(hinstance, PCWSTR(1 as *const u16))
                .unwrap_or_else(|_| LoadIconW(HINSTANCE::default(), windows::Win32::UI::WindowsAndMessaging::IDI_APPLICATION).unwrap());
            let h_icon_paused = LoadIconW(hinstance, PCWSTR(2 as *const u16))
                .unwrap_or(h_icon_active);

            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: h_icon_active,
                ..Default::default()
            };
            set_tooltip(&mut nid, "Teams Caffeine: Active (F15 every 30s)");

            let _ = Shell_NotifyIconW(NIM_ADD, &nid);

            let _ = STATE.set(Mutex::new(AppState {
                active: true,
                h_icon_active,
                h_icon_paused,
                nid,
            }));

            // Set up periodic 30s timer
            SetTimer(hwnd, TIMER_KEEP_AWAKE, KEEP_AWAKE_INTERVAL_MS, None);

            // Initial keep-awake kick
            apply_execution_state(true);
            send_f15_key();

            LRESULT(0)
        }

        WM_TIMER => {
            if wparam.0 == TIMER_KEEP_AWAKE {
                let is_active = STATE
                    .get()
                    .and_then(|s| s.lock().ok())
                    .map(|s| s.active)
                    .unwrap_or(false);

                if is_active {
                    apply_execution_state(true);
                    send_f15_key();
                }
            }
            LRESULT(0)
        }

        WM_TRAYICON => {
            let event = lparam.0 as u32;
            match event {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => {
                    // Left-click / double-click toggles active status
                    toggle_active(hwnd);
                }
                WM_RBUTTONUP => {
                    // Context Menu
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);

                    let hmenu = CreatePopupMenu().unwrap_or(HMENU::default());

                    let is_active = STATE
                        .get()
                        .and_then(|s| s.lock().ok())
                        .map(|s| s.active)
                        .unwrap_or(true);
                    let is_autostart = is_autostart_registered();

                    let title_text = if is_active {
                        w!("Teams Caffeine: Active")
                    } else {
                        w!("Teams Caffeine: Paused")
                    };

                    let _ = AppendMenuW(hmenu, MF_STRING | MF_DISABLED | MF_GRAYED, ID_TRAY_TITLE, title_text);
                    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());

                    let toggle_flag = if is_active { MF_CHECKED } else { MF_UNCHECKED };
                    let _ = AppendMenuW(hmenu, MF_STRING | toggle_flag, ID_TRAY_TOGGLE, w!("Active (Keep Awake)"));

                    let autostart_flag = if is_autostart { MF_CHECKED } else { MF_UNCHECKED };
                    let _ = AppendMenuW(hmenu, MF_STRING | autostart_flag, ID_TRAY_AUTOSTART, w!("Start with Windows"));

                    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                    let _ = AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, w!("Exit"));

                    let _ = SetForegroundWindow(hwnd);
                    let _ = TrackPopupMenuEx(
                        hmenu,
                        (TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON).0,
                        pt.x,
                        pt.y,
                        hwnd,
                        None,
                    );
                    let _ = DestroyMenu(hmenu);
                    let _ = PostMessageW(hwnd, WM_NULL, WPARAM(0), LPARAM(0));
                }
                _ => {}
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let cmd_id = (wparam.0 & 0xFFFF) as usize;
            match cmd_id {
                ID_TRAY_TOGGLE => {
                    toggle_active(hwnd);
                }
                ID_TRAY_AUTOSTART => {
                    let currently_enabled = is_autostart_registered();
                    let _ = set_autostart_registered(!currently_enabled);
                }
                ID_TRAY_EXIT => {
                    let _ = DestroyWindow(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            // Restore normal power state and remove tray icon
            apply_execution_state(false);

            if let Some(state_lock) = STATE.get() {
                if let Ok(state) = state_lock.lock() {
                    let _ = Shell_NotifyIconW(NIM_DELETE, &state.nid);
                }
            }

            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => {
            // Re-add tray icon if Windows Explorer / Taskbar restarted
            if let Some(&taskbar_msg) = WM_TASKBARCREATED.get() {
                if taskbar_msg != 0 && msg == taskbar_msg {
                    if let Some(state_lock) = STATE.get() {
                        if let Ok(state) = state_lock.lock() {
                            let _ = Shell_NotifyIconW(NIM_ADD, &state.nid);
                        }
                    }
                    return LRESULT(0);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }
}

fn main() {
    unsafe {
        // Prevent running multiple instances
        let mutex_name = w!("Local\\TeamsCaffeine_SingleInstance_Mutex");
        let mutex = CreateMutexW(None, true, mutex_name);
        let mutex_handle = match mutex {
            Ok(h) => {
                if windows::Win32::Foundation::GetLastError() == windows::Win32::Foundation::ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(h);
                    return;
                }
                h
            }
            Err(_) => return,
        };

        // Register TaskbarCreated message to survive Explorer restarts
        let taskbar_msg = RegisterWindowMessageW(w!("TaskbarCreated"));
        let _ = WM_TASKBARCREATED.set(taskbar_msg);

        let hinstance = HINSTANCE(GetModuleHandleW(None).unwrap_or_default().0);
        let class_name = w!("TeamsCaffeineTrayClass");

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Teams Caffeine"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            HWND::default(),
            HMENU::default(),
            hinstance,
            None,
        );

        if hwnd.is_err() {
            let _ = CloseHandle(mutex_handle);
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = CloseHandle(mutex_handle);
    }
}
