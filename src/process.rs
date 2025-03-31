use std::marker::PhantomData;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
use windows::core::Result;

pub type X32 = u32;
pub type X64 = u64;
pub type Auto = usize;

pub trait Arch{}

impl Arch for X32{}
impl Arch for X64{}
impl Arch for Auto{}

/// 打开进程
pub fn open_process(pid:u32) -> Result<HANDLE> {
    unsafe {
        OpenProcess(PROCESS_ALL_ACCESS, false, pid)
    }
}

pub struct ProcessBlock<A:Arch>{
    handle: HANDLE,
    marker: PhantomData<A>,
}

impl<A:Arch> ProcessBlock<A> {
    pub fn from(handle: HANDLE) -> Self{
        Self{ handle, marker: Default::default() }
    }
}

pub trait ToProcessBlock{
    fn into_process_block<A:Arch>(self)-> ProcessBlock<A>;
}

impl ToProcessBlock for HANDLE{
    fn into_process_block<A:Arch>(self) -> ProcessBlock<A> {
        ProcessBlock::from(self)
    }
}