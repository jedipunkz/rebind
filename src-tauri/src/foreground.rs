#[cfg(windows)]
mod imp {
    use std::path::Path;

    use windows::{
        Win32::{
            Foundation::{CloseHandle, MAX_PATH},
            System::Threading::{
                OpenProcess, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
                QueryFullProcessImageNameW,
            },
            UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
        core::PWSTR,
    };

    pub fn active_exe_name() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }

            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buffer = vec![0u16; MAX_PATH as usize];
            let mut size = buffer.len() as u32;
            let result = QueryFullProcessImageNameW(
                process,
                PROCESS_NAME_FORMAT(0),
                PWSTR(buffer.as_mut_ptr()),
                &mut size,
            );
            let _ = CloseHandle(process);
            result.ok()?;

            let path = String::from_utf16_lossy(&buffer[..size as usize]);
            Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        }
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn active_exe_name() -> Option<String> {
        None
    }
}

#[cfg(windows)]
pub use imp::active_exe_name;
