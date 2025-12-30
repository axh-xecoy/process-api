use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{
    GetCurrentProcess, IsWow64Process, OpenProcess, PROCESS_ALL_ACCESS,
};
use windows::core::{BOOL, Error, Result};

pub mod list;
pub use list::{find_processes_by_name, list_processes, ProcessInfo};

pub fn open_process_handle(pid: u32) -> Result<HANDLE> {
    unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PointerWidth {
    X32,
    X64,
}

impl PointerWidth {
    pub fn bytes(self) -> usize {
        match self {
            PointerWidth::X32 => 4,
            PointerWidth::X64 => 8,
        }
    }
}

pub fn detect_pointer_width(handle: HANDLE) -> Result<PointerWidth> {
    let mut target_is_wow64 = BOOL(0);
    unsafe { IsWow64Process(handle, &mut target_is_wow64) }?;
    let target_is_wow64 = target_is_wow64.as_bool();

    if cfg!(target_pointer_width = "64") {
        return Ok(if target_is_wow64 {
            PointerWidth::X32
        } else {
            PointerWidth::X64
        });
    }

    let mut current_is_wow64 = BOOL(0);
    unsafe { IsWow64Process(GetCurrentProcess(), &mut current_is_wow64) }?;
    let os_is_64 = current_is_wow64.as_bool();

    if !os_is_64 {
        return Ok(PointerWidth::X32);
    }

    if target_is_wow64 {
        Ok(PointerWidth::X32)
    } else {
        Err(Error::new(
            windows::core::HRESULT(0x80070057u32 as i32),
            "32-bit build cannot operate on 64-bit target process",
        ))
    }
}

pub struct ProcessBlock {
    handle: HANDLE,
    pointer_width: PointerWidth,
}

impl ProcessBlock {
    pub fn from(handle: HANDLE, pointer_width: PointerWidth) -> Self {
        Self {
            handle,
            pointer_width,
        }
    }

    pub fn pointer_width(&self) -> PointerWidth {
        self.pointer_width
    }

    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}

pub trait ToProcessBlock {
    fn into_process_block(self) -> Result<ProcessBlock>;
}

impl ToProcessBlock for HANDLE {
    fn into_process_block(self) -> Result<ProcessBlock> {
        let pointer_width = detect_pointer_width(self)?;
        Ok(ProcessBlock::from(self, pointer_width))
    }
}

pub fn open_process(pid: u32) -> Result<ProcessBlock> {
    let handle = open_process_handle(pid)?;
    handle.into_process_block()
}

