use std::mem::{size_of, zeroed};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_RETURN,
};

const VK_V: u16 = 0x56;

fn keyboard_input(key: u16, flags: u32) -> INPUT {
    let mut input: INPUT = unsafe { zeroed() };
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous.ki = KEYBDINPUT {
        wVk: key,
        wScan: 0,
        dwFlags: flags,
        time: 0,
        dwExtraInfo: 0,
    };
    input
}

fn send(inputs: &[INPUT]) -> Result<(), String> {
    let sent = unsafe {
        SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            size_of::<INPUT>() as i32,
        )
    };
    if sent == inputs.len() as u32 {
        Ok(())
    } else {
        Err(format!(
            "Windows 只接受了 {sent}/{} 个键盘事件",
            inputs.len()
        ))
    }
}

pub fn press_enter() -> Result<(), String> {
    send(&[
        keyboard_input(VK_RETURN, 0),
        keyboard_input(VK_RETURN, KEYEVENTF_KEYUP),
    ])
}

pub fn press_ctrl_v() -> Result<(), String> {
    send(&[
        keyboard_input(VK_CONTROL, 0),
        keyboard_input(VK_V, 0),
        keyboard_input(VK_V, KEYEVENTF_KEYUP),
        keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
    ])
}
