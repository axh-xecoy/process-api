use crate::{Arch, ProcessBlock};
use std::error::Error;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::size_of;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};

/// 内存空间
pub struct MemoryBlock<T:Default, A:Arch>{
    handle: HANDLE,
    address_offset: Vec<usize>,
    _marker: PhantomData<T>,
    _phantom: PhantomData<A>,
}

impl<A:Arch> ProcessBlock<A>{
    pub fn memory_block<T:Default>(&self, address_offset: Vec<usize>) -> MemoryBlock<T,A> {
        MemoryBlock::new(
            self.handle,
            address_offset
        )
    }
}
unsafe impl<T:Default, A:Arch> Send for MemoryBlock<T, A>{}

impl<T:Default, A:Arch> MemoryBlock<T, A>{
    pub fn new(handle: HANDLE, address_offset:Vec<usize>) -> Self{
        if address_offset.len() == 0 {
            panic!("内存地址不能为空！");
        }
        Self{ handle, address_offset, _marker:Default::default(), _phantom: Default::default() }
    }

    ///从内存块读取内容
    pub fn read(&self) -> Result<T, Box<dyn Error>>{
        unsafe{
            read_process_memory::<_,A>(self.handle,&self.address_offset)
        }
    }

    ///向内存块写入内容
    pub fn write(&self, data:T) -> Result<(), Box<dyn Error>>{
        unsafe{
            write_process_memory::<_,A>(self.handle,&self.address_offset,data)
        }
    }

    /// 根据相对偏移量读取内存中的数据
    pub fn read_with_offset(&self, offset: usize) -> Result<T, Box<dyn Error>> {
        let mut new_offsets = self.address_offset.clone();
        let last_index = new_offsets.len() - 1;
        new_offsets[last_index] += offset;
        unsafe {
            read_process_memory::<_,A>(self.handle, &new_offsets)
        }
    }

    /// 根据相对偏移量写入数据到内存
    pub fn write_with_offset(&self, offset: usize, data: T) -> Result<(), Box<dyn Error>> {
        let mut new_offsets = self.address_offset.clone();
        let last_index = new_offsets.len() - 1;
        new_offsets[last_index] += offset;
        unsafe {
            write_process_memory::<_,A>(self.handle, &new_offsets, data)
        }
    }
}

/// 解析多级指针，返回最终的地址
unsafe fn resolve_multilevel_pointer<A>(
    handle: HANDLE,
    address_offset: &[usize],
) -> Result<usize, Box<dyn Error>>
where
    A: Arch,
{
    if address_offset.is_empty() {
        panic!("不能传入空地址");
    }
    let size = size_of::<A>();

    let mut address = address_offset[0];

    // 如果地址偏移量多于一个，逐级读取指针
    if address_offset.len() > 1 {
        for &offset in &address_offset[1..] {
            let mut address_temp: usize = 0;
            ReadProcessMemory(
                handle,
                address as *const c_void,
                &mut address_temp as *mut usize as *mut c_void,
                size,
                None,
            )?;
            address = address_temp + offset;
        }
    }
    Ok(address)
}

/// 读取进程内存
pub unsafe fn read_process_memory<T,A>(
    handle: HANDLE,
    address_offset: &[usize], // 改为借用
) -> Result<T, Box<dyn Error>>
where
    T: Default, // 确保 T 可以被初始化
    A: Arch,
{
    let t_size = size_of::<T>();
    let mut result_value: T = Default::default(); // 初始化 T
    let address = resolve_multilevel_pointer::<A>(handle, address_offset)?; // 传递借用
    // 读取最终的值
    ReadProcessMemory(
        handle,
        address as *const c_void,
        &mut result_value as *mut T as *mut c_void,
        t_size,
        None,
    )?;
    Ok(result_value)
}

/// 写入进程内存
pub unsafe fn write_process_memory<T,A>(
    handle: HANDLE,
    address_offset: &[usize], // 改为借用
    value: T,
) -> Result<(), Box<dyn Error>>
where
    A: Arch,
{
    let t_size = size_of::<T>();
    let address = resolve_multilevel_pointer::<A>(handle, address_offset)?; // 传递借用
    // 写入值
    WriteProcessMemory(
        handle,
        address as *const c_void,
        &value as *const T as *const c_void,
        t_size,
        None,
    )?;
    Ok(())
}