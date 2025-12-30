pub mod process;
pub mod memory;
pub mod window;

pub use process::{
    detect_pointer_width, open_process, open_process_handle, PointerWidth, ProcessBlock,
    ToProcessBlock,
};
pub use process::{find_processes_by_name, list_processes, ProcessInfo};

pub use memory::{
    read_process_memory_with_pointer_size, write_process_memory_with_pointer_size, MemoryBlock,
};

pub use window::{
    bring_to_foreground, class_name, close_window, find_window, first_visible_window_by_pid,
    first_visible_window_by_process_name, get_foreground_window, hide_window, is_window,
    is_window_visible, maximize_window, minimize_window, move_window, restore_window,
    send_hotkey, send_hotkey_to_window, set_window_topmost, show_window, window_rect,
    window_text, windows_by_pid, windows_by_process_name,
};

#[cfg(test)]
mod tests {
    use crate::{list_processes, open_process, open_process_handle, process, window};
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_F11;
    use windows::Win32::System::Threading::GetCurrentProcessId;
    #[test]
    #[ignore]
    fn it_works() {
        let pid = 41256;
        let _handle = open_process_handle(pid).unwrap();

        let process = open_process(pid).unwrap();
        let memory_block = process.memory_block::<bool>(vec![0x6A9EC0, 0x5AC]);
        memory_block.write(true).unwrap();

        let hwnds = window::windows_by_pid(pid).unwrap();
        let hwnd = hwnds
            .into_iter()
            .find(|h| window::is_window_visible(*h))
            .unwrap();

        window::restore_window(hwnd).unwrap();
        window::set_window_topmost(hwnd, true).unwrap();
        window::bring_to_foreground(hwnd).unwrap();
        window::send_hotkey_to_window(hwnd, &[VK_F11]).unwrap();
    }

    #[test]
    fn list_processes_non_empty() {
        let processes = list_processes().unwrap();
        assert!(!processes.is_empty());

        let current_pid = unsafe { GetCurrentProcessId() };
        let current_exe = std::env::current_exe().unwrap();
        let current_name = current_exe.file_name().unwrap().to_string_lossy().to_string();

        let matches = process::find_processes_by_name(&current_name).unwrap();
        assert!(matches.iter().any(|p| p.pid == current_pid));
    }

}
