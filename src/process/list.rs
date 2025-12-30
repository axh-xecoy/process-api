use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub exe_name: String,
}

fn wide_cstr_to_string(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(Error::from_win32());
    }

    let mut processes = Vec::new();

    let mut entry = PROCESSENTRY32W::default();
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

    unsafe { Process32FirstW(snapshot, &mut entry) }?;
    loop {
        processes.push(ProcessInfo {
            pid: entry.th32ProcessID,
            exe_name: wide_cstr_to_string(&entry.szExeFile),
        });
        if unsafe { Process32NextW(snapshot, &mut entry) }.is_err() {
            break;
        }
    }

    let _ = unsafe { CloseHandle(snapshot) };
    Ok(processes)
}

pub fn find_processes_by_name(name: &str) -> Result<Vec<ProcessInfo>> {
    let target = name.to_ascii_lowercase();
    let target_exe = if target.ends_with(".exe") {
        None
    } else {
        Some(format!("{target}.exe"))
    };
    let processes = list_processes()?;
    Ok(processes
        .into_iter()
        .filter(|p| {
            let exe = p.exe_name.to_ascii_lowercase();
            exe == target || target_exe.as_ref().is_some_and(|v| exe == *v)
        })
        .collect())
}

