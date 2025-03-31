use std::error::Error;
use std::mem::size_of;
use std::ffi::c_void;
use std::marker::PhantomData;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};

/// 内存读取块
pub struct DataBlock<T:Default>{
    handle: HANDLE,
    address_offset: Vec<usize>,
    _marker: PhantomData<T>,
}

unsafe impl<T:Default> Send for DataBlock<T>{}
impl<T:Default> DataBlock<T>{
    pub fn new(handle: HANDLE, address_offset:Vec<usize>) -> Self{
        if address_offset.len() == 0 {
            panic!("内存地址不能为空！");
        }
        Self{ handle, address_offset, _marker:Default::default() }
    }

    ///从数组块读取内容
    pub fn read(&self) -> Result<T, Box<dyn Error>>{
        unsafe{
            read_process_memory(self.handle,&self.address_offset)
        }
    }

    ///向数组块写入内容
    pub fn write(&self, data:T) -> Result<(), Box<dyn Error>>{
        unsafe{
            write_process_memory(self.handle,&self.address_offset,data)
        }
    }

    /// 根据相对偏移量读取内存中的数据
    pub fn read_with_offset(&self, offset: usize) -> Result<T, Box<dyn Error>> {
        let mut new_offsets = self.address_offset.clone();
        let last_index = new_offsets.len() - 1;
        new_offsets[last_index] += offset;
        unsafe {
            read_process_memory(self.handle, &new_offsets)
        }
    }

    /// 根据相对偏移量写入数据到内存
    pub fn write_with_offset(&self, offset: usize, data: T) -> Result<(), Box<dyn Error>> {
        let mut new_offsets = self.address_offset.clone();
        let last_index = new_offsets.len() - 1;
        new_offsets[last_index] += offset;
        unsafe {
            write_process_memory(self.handle, &new_offsets, data)
        }
    }
}

/// 解析多级指针，返回最终的地址
unsafe fn resolve_multilevel_pointer(
    handle: HANDLE,
    address_offset: &[usize],
) -> Result<usize, Box<dyn Error>> {
    if address_offset.is_empty() {
        panic!("不能传入空地址");
    }
    let size = size_of::<usize>();
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
pub unsafe fn read_process_memory<T>(
    handle: HANDLE,
    address_offset: &[usize], // 改为借用
) -> Result<T, Box<dyn Error>>
where
    T: Default, // 确保 T 可以被初始化
{
    let t_size = size_of::<T>();
    let mut result_value: T = Default::default(); // 初始化 T
    let address = resolve_multilevel_pointer(handle, address_offset)?; // 传递借用
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
pub unsafe fn write_process_memory<T>(
    handle: HANDLE,
    address_offset: &[usize], // 改为借用
    value: T,
) -> Result<(), Box<dyn Error>> {
    let t_size = size_of::<T>();
    let address = resolve_multilevel_pointer(handle, address_offset)?; // 传递借用
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