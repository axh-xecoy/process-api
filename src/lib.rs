pub mod memory;
pub mod window;

#[cfg(test)]
mod tests {
    use crate::{open_process, open_process_handle, window};
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_F11;
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
}

include!("process.rs");