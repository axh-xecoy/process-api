use crate::{PointerWidth, ProcessBlock};
use std::error::Error;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};

/// 内存空间
pub struct MemoryBlock<T: Default> {
    handle: HANDLE,
    address_offset: Vec<usize>,
    pointer_width: PointerWidth,
    _marker: PhantomData<T>,
}

impl ProcessBlock {
    pub fn memory_block<T: Default>(&self, address_offset: Vec<usize>) -> MemoryBlock<T> {
        MemoryBlock::new(self.handle(), address_offset, self.pointer_width())
    }
}

unsafe impl<T: Default> Send for MemoryBlock<T> {}

impl<T: Default> MemoryBlock<T> {
    pub fn new(handle: HANDLE, address_offset: Vec<usize>, pointer_width: PointerWidth) -> Self {
        if address_offset.is_empty() {
            panic!("内存地址不能为空！");
        }
        Self {
            handle,
            address_offset,
            pointer_width,
            _marker: Default::default(),
        }
    }

    ///从内存块读取内容
    pub fn read(&self) -> Result<T, Box<dyn Error>> {
        unsafe {
            read_process_memory_with_pointer_size::<_>(
                self.handle,
                &self.address_offset,
                self.pointer_width.bytes(),
            )
        }
    }

    ///向内存块写入内容
    pub fn write(&self, data: T) -> Result<(), Box<dyn Error>> {
        unsafe {
            write_process_memory_with_pointer_size::<_>(
                self.handle,
                &self.address_offset,
                self.pointer_width.bytes(),
                data,
            )
        }
    }

    /// 根据相对偏移量读取内存中的数据
    pub fn read_with_offset(&self, offset: usize) -> Result<T, Box<dyn Error>> {
        let mut new_offsets = self.address_offset.clone();
        let last_index = new_offsets.len() - 1;
        new_offsets[last_index] += offset;
        unsafe {
            read_process_memory_with_pointer_size::<_>(
                self.handle,
                &new_offsets,
                self.pointer_width.bytes(),
            )
        }
    }

    /// 根据相对偏移量写入数据到内存
    pub fn write_with_offset(&self, offset: usize, data: T) -> Result<(), Box<dyn Error>> {
        let mut new_offsets = self.address_offset.clone();
        let last_index = new_offsets.len() - 1;
        new_offsets[last_index] += offset;
        unsafe {
            write_process_memory_with_pointer_size::<_>(
                self.handle,
                &new_offsets,
                self.pointer_width.bytes(),
                data,
            )
        }
    }
}

unsafe fn resolve_multilevel_pointer_with_size(
    handle: HANDLE,
    address_offset: &[usize],
    pointer_size: usize,
) -> Result<usize, Box<dyn Error>> {
    if address_offset.is_empty() {
        panic!("不能传入空地址");
    }
    if pointer_size != 4 && pointer_size != 8 {
        panic!("不支持的指针大小");
    }

    let mut address = address_offset[0];

    if address_offset.len() > 1 {
        for &offset in &address_offset[1..] {
            let mut address_temp: usize = 0;
            ReadProcessMemory(
                handle,
                address as *const c_void,
                &mut address_temp as *mut usize as *mut c_void,
                pointer_size,
                None,
            )?;
            address = address_temp + offset;
        }
    }
    Ok(address)
}

pub unsafe fn read_process_memory_with_pointer_size<T>(
    handle: HANDLE,
    address_offset: &[usize],
    pointer_size: usize,
) -> Result<T, Box<dyn Error>>
where
    T: Default,
{
    let t_size = size_of::<T>();
    let mut result_value: T = Default::default();
    let address = resolve_multilevel_pointer_with_size(handle, address_offset, pointer_size)?;
    ReadProcessMemory(
        handle,
        address as *const c_void,
        &mut result_value as *mut T as *mut c_void,
        t_size,
        None,
    )?;
    Ok(result_value)
}

pub unsafe fn write_process_memory_with_pointer_size<T>(
    handle: HANDLE,
    address_offset: &[usize],
    pointer_size: usize,
    value: T,
) -> Result<(), Box<dyn Error>> {
    let t_size = size_of::<T>();
    let address = resolve_multilevel_pointer_with_size(handle, address_offset, pointer_size)?;
    WriteProcessMemory(
        handle,
        address as *const c_void,
        &value as *const T as *const c_void,
        t_size,
        None,
    )?;
    Ok(())
}
