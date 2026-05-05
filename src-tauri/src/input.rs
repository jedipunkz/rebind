use crate::config::{BindingAction, KeyChord, Modifiers};

#[cfg(windows)]
mod imp {
    use crate::config::{Key, KeyChord, Modifiers};
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY,
        VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT, VK_LWIN,
        VK_MENU, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
    };

    pub fn send_chord(chord: &KeyChord) -> bool {
        let mut inputs = Vec::new();
        push_modifiers(&mut inputs, chord.modifiers, false);
        inputs.push(key_input(vk_for_key(&chord.key), false));
        inputs.push(key_input(vk_for_key(&chord.key), true));
        push_modifiers(&mut inputs, chord.modifiers, true);

        unsafe {
            SendInput(
                &inputs,
                std::mem::size_of::<INPUT>()
                    .try_into()
                    .expect("INPUT size fits i32"),
            ) == inputs.len() as u32
        }
    }

    pub fn send_with_neutralized_modifiers(chords: &[KeyChord], source: Modifiers) -> bool {
        let mut prefix = Vec::new();
        push_modifiers(&mut prefix, source, true);
        let mut suffix = Vec::new();
        push_modifiers(&mut suffix, source, false);

        unsafe {
            if !prefix.is_empty()
                && SendInput(
                    &prefix,
                    std::mem::size_of::<INPUT>()
                        .try_into()
                        .expect("INPUT size fits i32"),
                ) != prefix.len() as u32
            {
                return false;
            }
        }

        let sent = chords.iter().all(send_chord);

        unsafe {
            if !suffix.is_empty() {
                let _ = SendInput(
                    &suffix,
                    std::mem::size_of::<INPUT>()
                        .try_into()
                        .expect("INPUT size fits i32"),
                );
            }
        }

        sent
    }

    pub fn send_paste(source: Modifiers) -> bool {
        let released = Modifiers {
            ctrl: false,
            shift: source.shift,
            alt: source.alt,
            win: source.win,
        };
        let press_ctrl = !source.ctrl;

        let mut inputs = Vec::new();
        push_modifiers(&mut inputs, released, true);
        if press_ctrl {
            inputs.push(key_input(VK_CONTROL, false));
        }
        inputs.push(key_input(VIRTUAL_KEY('V' as u16), false));
        inputs.push(key_input(VIRTUAL_KEY('V' as u16), true));
        if press_ctrl {
            inputs.push(key_input(VK_CONTROL, true));
        }
        push_modifiers(&mut inputs, released, false);

        unsafe {
            SendInput(
                &inputs,
                std::mem::size_of::<INPUT>()
                    .try_into()
                    .expect("INPUT size fits i32"),
            ) == inputs.len() as u32
        }
    }

    fn push_modifiers(inputs: &mut Vec<INPUT>, modifiers: Modifiers, key_up: bool) {
        if modifiers.win {
            inputs.push(key_input(VK_LWIN, key_up));
        }
        if modifiers.alt {
            inputs.push(key_input(VK_MENU, key_up));
        }
        if modifiers.shift {
            inputs.push(key_input(VK_SHIFT, key_up));
        }
        if modifiers.ctrl {
            inputs.push(key_input(VK_CONTROL, key_up));
        }
    }

    fn key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if key_up {
                        KEYEVENTF_KEYUP
                    } else {
                        Default::default()
                    },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn vk_for_key(key: &Key) -> VIRTUAL_KEY {
        match key {
            Key::Char(ch) => VIRTUAL_KEY(ch.to_ascii_uppercase() as u16),
            Key::Home => VK_HOME,
            Key::End => VK_END,
            Key::Left => VK_LEFT,
            Key::Right => VK_RIGHT,
            Key::Up => VK_UP,
            Key::Down => VK_DOWN,
            Key::Backspace => VK_BACK,
            Key::Delete => VK_DELETE,
            Key::Escape => VK_ESCAPE,
            Key::Enter => VK_RETURN,
            Key::Tab => VK_TAB,
            Key::Space => VK_SPACE,
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use crate::config::{KeyChord, Modifiers};

    pub fn send_chord(_chord: &KeyChord) -> bool {
        false
    }

    pub fn send_with_neutralized_modifiers(_chords: &[KeyChord], _source: Modifiers) -> bool {
        false
    }

    pub fn send_paste(_source: Modifiers) -> bool {
        false
    }
}

pub fn send_action(action: &BindingAction, source_modifiers: Modifiers) -> bool {
    match action {
        BindingAction::KeySequence { chords } => {
            imp::send_with_neutralized_modifiers(chords, source_modifiers)
        }
        BindingAction::Paste => imp::send_paste(source_modifiers),
    }
}
