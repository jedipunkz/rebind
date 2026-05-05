use std::sync::Arc;

use thiserror::Error;

use crate::app::AppState;

#[derive(Debug, Error)]
pub enum HookError {
    #[cfg(windows)]
    #[error("failed to install keyboard hook")]
    InstallFailed,
}

#[cfg(windows)]
mod imp {
    use std::{
        sync::{
            Arc, OnceLock,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
        time::Duration,
    };

    use windows::Win32::{
        Foundation::{LPARAM, LRESULT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT,
                VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SHIFT,
            },
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED,
                MSG, SetWindowsHookExW, TranslateMessage, WH_KEYBOARD_LL, WM_KEYDOWN,
                WM_SYSKEYDOWN,
            },
        },
    };

    use super::HookError;
    use crate::{
        app::AppState,
        config::{Key, KeyChord, Modifiers},
        foreground, input,
    };

    static STATE: OnceLock<Arc<AppState>> = OnceLock::new();
    static INSTALLED: AtomicBool = AtomicBool::new(false);

    pub fn install(state: Arc<AppState>) -> Result<(), HookError> {
        let _ = STATE.set(state);
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), None, 0);
            match hook {
                Ok(_hook) => {
                    INSTALLED.store(true, Ordering::Release);
                    let _ = tx.send(true);
                }
                Err(error) => {
                    tracing::error!("SetWindowsHookExW failed: {error}");
                    let _ = tx.send(false);
                    return;
                }
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });

        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(true) => Ok(()),
            _ => Err(HookError::InstallFailed),
        }
    }

    pub fn uninstall() {}

    unsafe extern "system" fn callback(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
        if n_code < 0 {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }

        let is_key_down = w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN;
        if !is_key_down {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }

        let event = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        if event.flags.contains(LLKHF_INJECTED) {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }

        if handle_key(event.vkCode) {
            LRESULT(1)
        } else {
            unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
        }
    }

    fn handle_key(vk_code: u32) -> bool {
        let Some(state) = STATE.get() else {
            return false;
        };
        if !state.is_enabled() {
            return false;
        }

        let config = state.config();
        if let Some(exe_name) = foreground::active_exe_name() {
            if config.should_ignore_app(&exe_name) {
                return false;
            }
        }

        let Some(chord) = chord_from_vk(vk_code) else {
            return false;
        };
        let Some(action) = config.action_for(&chord) else {
            return false;
        };

        input::send_action(action, chord.modifiers)
    }

    fn chord_from_vk(vk_code: u32) -> Option<KeyChord> {
        let key = key_from_vk(vk_code)?;
        Some(KeyChord {
            modifiers: current_modifiers(vk_code),
            key,
        })
    }

    fn key_from_vk(vk_code: u32) -> Option<Key> {
        match vk_code {
            0x30..=0x39 => Some(Key::Char(char::from_u32(vk_code)?)),
            0x41..=0x5A => Some(Key::Char(char::from_u32(vk_code)?.to_ascii_lowercase())),
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_HOME.0 as u32 => {
                Some(Key::Home)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_END.0 as u32 => {
                Some(Key::End)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_LEFT.0 as u32 => {
                Some(Key::Left)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_RIGHT.0 as u32 => {
                Some(Key::Right)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_UP.0 as u32 => {
                Some(Key::Up)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_DOWN.0 as u32 => {
                Some(Key::Down)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_BACK.0 as u32 => {
                Some(Key::Backspace)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_DELETE.0 as u32 => {
                Some(Key::Delete)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE.0 as u32 => {
                Some(Key::Escape)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN.0 as u32 => {
                Some(Key::Enter)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_TAB.0 as u32 => {
                Some(Key::Tab)
            }
            code if code == windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE.0 as u32 => {
                Some(Key::Space)
            }
            _ => None,
        }
    }

    fn current_modifiers(current_vk: u32) -> Modifiers {
        Modifiers {
            ctrl: is_pressed(VK_CONTROL) || is_pressed(VK_LCONTROL) || is_pressed(VK_RCONTROL),
            shift: is_pressed(VK_SHIFT) || is_pressed(VK_LSHIFT) || is_pressed(VK_RSHIFT),
            alt: is_pressed(VK_MENU) || is_pressed(VK_LMENU) || is_pressed(VK_RMENU),
            win: is_pressed(VK_LWIN) || is_pressed(VK_RWIN),
        }
        .without_current_modifier(current_vk)
    }

    trait WithoutCurrentModifier {
        fn without_current_modifier(self, current_vk: u32) -> Self;
    }

    impl WithoutCurrentModifier for Modifiers {
        fn without_current_modifier(mut self, current_vk: u32) -> Self {
            match VIRTUAL_KEY(current_vk as u16) {
                VK_CONTROL | VK_LCONTROL | VK_RCONTROL => self.ctrl = false,
                VK_SHIFT | VK_LSHIFT | VK_RSHIFT => self.shift = false,
                VK_MENU | VK_LMENU | VK_RMENU => self.alt = false,
                VK_LWIN | VK_RWIN => self.win = false,
                _ => {}
            }
            self
        }
    }

    fn is_pressed(key: VIRTUAL_KEY) -> bool {
        unsafe { GetAsyncKeyState(key.0 as i32) < 0 }
    }
}

#[cfg(not(windows))]
mod imp {
    use std::sync::Arc;

    use super::HookError;
    use crate::app::AppState;

    pub fn install(_state: Arc<AppState>) -> Result<(), HookError> {
        Ok(())
    }

    pub fn uninstall() {}
}

pub fn install(state: Arc<AppState>) -> Result<(), HookError> {
    imp::install(state)
}

pub fn uninstall() {
    imp::uninstall();
}
