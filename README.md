# process-api

一个面向 Windows 的 Rust 工具库，提供：

- 打开指定 PID 进程并自动识别目标进程指针位宽（32/64）
- 按多级指针偏移链读写目标进程内存
- 获取窗口句柄、窗口基础操作（前置/置顶/移动/关闭等）
- 发送快捷键（基于 `SendInput`）

## 环境

- Windows
- Rust edition 2021

依赖使用 `windows` crate 调用 Win32 API。

## 安装

如果你把它当作 workspace 里的 crate 引用：

```toml
[dependencies]
process-api = { path = "../process-api" }
```

## 用法

### 打开进程并写内存（多级指针偏移）

`open_process(pid)` 会返回 `ProcessBlock`，内部会用 WinAPI 自动检测目标进程是 32 位还是 64 位，并按对应的指针宽度解析多级指针。

```rust
use process_api::open_process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 41256;
    let process = open_process(pid)?;

    let flag = process.memory_block::<bool>(vec![0x6A9EC0, 0x5AC]);
    flag.write(true)?;

    Ok(())
}
```

### 获取目标进程窗口并操作窗口 + 发送 F11

```rust
use process_api::{open_process, window};
use windows::Win32::UI::Input::KeyboardAndMouse::VK_F11;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pid = 41256;

    let process = open_process(pid)?;
    process
        .memory_block::<bool>(vec![0x6A9EC0, 0x5AC])
        .write(true)?;

    let hwnd = window::windows_by_pid(pid)?
        .into_iter()
        .find(|h| window::is_window_visible(*h))
        .ok_or("no visible window")?;

    window::restore_window(hwnd)?;
    window::set_window_topmost(hwnd, true)?;
    window::bring_to_foreground(hwnd)?;
    window::send_hotkey_to_window(hwnd, &[VK_F11])?;

    Ok(())
}
```

## API 入口

- 进程与位宽
  - `open_process_handle(pid) -> HANDLE`
  - `open_process(pid) -> Result<ProcessBlock>`
  - `detect_pointer_width(handle) -> Result<PointerWidth>`
- 内存
  - `ProcessBlock::memory_block::<T>(offsets) -> MemoryBlock<T>`
  - `MemoryBlock::read / write / read_with_offset / write_with_offset`
- 窗口
  - `windows_by_pid(pid) -> Result<Vec<HWND>>`
  - `bring_to_foreground / set_window_topmost / restore_window / move_window / close_window ...`
  - `send_hotkey` / `send_hotkey_to_window`

## 注意事项

- 读写进程内存通常需要足够权限（例如管理员权限）。
- Windows 对“后台程序抢前台窗口”有系统限制，`bring_to_foreground` 不一定总能成功。
- `bool`/结构体等类型会按 Rust 内存布局直接读写；请确保目标进程内存布局与类型匹配。

