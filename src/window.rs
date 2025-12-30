use std::mem::size_of;
use windows::core::{BOOL, Error, PCWSTR, Result};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowW, GetClassNameW, GetForegroundWindow, GetWindowRect,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
    MoveWindow, PostMessageW, SetForegroundWindow, SetWindowPos, ShowWindow, HWND_NOTOPMOST,
    HWND_TOPMOST, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE,
    WM_CLOSE,
};

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

pub fn get_foreground_window() -> Option<HWND> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        None
    } else {
        Some(hwnd)
    }
}

pub fn is_window(hwnd: HWND) -> bool {
    unsafe { IsWindow(Some(hwnd)) }.as_bool()
}

pub fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd) }.as_bool()
}

pub fn find_window(class_name: Option<&str>, title: Option<&str>) -> Option<HWND> {
    let class_wide = class_name.map(wide_null);
    let title_wide = title.map(wide_null);

    let class_ptr = class_wide
        .as_ref()
        .map(|v| PCWSTR(v.as_ptr()))
        .unwrap_or(PCWSTR::null());

    let title_ptr = title_wide
        .as_ref()
        .map(|v| PCWSTR(v.as_ptr()))
        .unwrap_or(PCWSTR::null());

    let hwnd = unsafe { FindWindowW(class_ptr, title_ptr) }.ok()?;
    if hwnd.0.is_null() { None } else { Some(hwnd) }
}

pub fn window_text(hwnd: HWND) -> Result<String> {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return Ok(String::new());
    }
    let mut buffer = vec![0u16; (len as usize) + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    if copied == 0 {
        return Ok(String::new());
    }
    Ok(String::from_utf16_lossy(&buffer[..(copied as usize)]))
}

pub fn class_name(hwnd: HWND) -> Result<String> {
    let mut buffer = vec![0u16; 256];
    let copied = unsafe { GetClassNameW(hwnd, &mut buffer) };
    if copied == 0 {
        return Ok(String::new());
    }
    Ok(String::from_utf16_lossy(&buffer[..(copied as usize)]))
}

pub fn window_rect(hwnd: HWND) -> Result<RECT> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect) }?;
    Ok(rect)
}

pub fn bring_to_foreground(hwnd: HWND) -> Result<()> {
    let ok = unsafe { SetForegroundWindow(hwnd) };
    if ok.as_bool() { Ok(()) } else { Err(Error::from_win32()) }
}

pub fn set_window_topmost(hwnd: HWND, topmost: bool) -> Result<()> {
    let insert_after = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
    unsafe { SetWindowPos(hwnd, Some(insert_after), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) }?;
    Ok(())
}

pub fn focus_window(hwnd: HWND) -> Result<()> {
    bring_to_foreground(hwnd)
}

pub fn show_window(hwnd: HWND) -> Result<()> {
    let _ = unsafe { ShowWindow(hwnd, SW_SHOW) };
    Ok(())
}

pub fn hide_window(hwnd: HWND) -> Result<()> {
    let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
    Ok(())
}

pub fn minimize_window(hwnd: HWND) -> Result<()> {
    let _ = unsafe { ShowWindow(hwnd, SW_MINIMIZE) };
    Ok(())
}

pub fn maximize_window(hwnd: HWND) -> Result<()> {
    let _ = unsafe { ShowWindow(hwnd, SW_MAXIMIZE) };
    Ok(())
}

pub fn restore_window(hwnd: HWND) -> Result<()> {
    let _ = unsafe { ShowWindow(hwnd, SW_RESTORE) };
    Ok(())
}

pub fn move_window(hwnd: HWND, x: i32, y: i32, width: i32, height: i32, repaint: bool) -> Result<()> {
    unsafe { MoveWindow(hwnd, x, y, width, height, repaint) }?;
    Ok(())
}

pub fn close_window(hwnd: HWND) -> Result<()> {
    unsafe { PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0)) }?;
    Ok(())
}

pub fn windows_by_pid(pid: u32) -> Result<Vec<HWND>> {
    struct Ctx {
        pid: u32,
        hwnds: Vec<HWND>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let ctx = unsafe { &mut *(lparam.0 as *mut Ctx) };
        let mut window_pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)) };
        if window_pid == ctx.pid {
            ctx.hwnds.push(hwnd);
        }
        BOOL(1)
    }

    let mut ctx = Box::new(Ctx { pid, hwnds: Vec::new() });
    let lparam = LPARAM((&mut *ctx as *mut Ctx) as isize);
    unsafe { EnumWindows(Some(enum_proc), lparam) }?;
    Ok(ctx.hwnds)
}

fn keyboard_input(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub fn send_hotkey(keys: &[VIRTUAL_KEY]) -> Result<()> {
    if keys.is_empty() {
        return Ok(());
    }

    let mut inputs: Vec<INPUT> = Vec::with_capacity(keys.len() * 2);
    for &vk in keys {
        inputs.push(keyboard_input(vk, KEYBD_EVENT_FLAGS(0)));
    }
    for &vk in keys.iter().rev() {
        inputs.push(keyboard_input(vk, KEYEVENTF_KEYUP));
    }

    let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(Error::from_win32())
    }
}

pub fn send_hotkey_to_window(hwnd: HWND, keys: &[VIRTUAL_KEY]) -> Result<()> {
    restore_window(hwnd)?;
    bring_to_foreground(hwnd)?;
    send_hotkey(keys)
}
