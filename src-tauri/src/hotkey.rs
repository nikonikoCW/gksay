use std::{thread, time::Duration};
use tauri::AppHandle;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_F3};

const POLL_INTERVAL: Duration = Duration::from_millis(8);

fn key_is_down(key: u16) -> bool {
    // GetAsyncKeyState 的最高位表示按键此刻是否处于按下状态。
    unsafe { GetAsyncKeyState(i32::from(key)) < 0 }
}

pub fn start(app: AppHandle) -> Result<(), String> {
    thread::Builder::new()
        .name("gksay-hotkey".into())
        .spawn(move || {
            let mut was_pressed = false;

            loop {
                let pressed = key_is_down(VK_CONTROL) && key_is_down(VK_F3);

                // 只响应从“未按下”到“按下”的边沿，长按不会反复触发。
                if pressed && !was_pressed {
                    crate::runner::toggle(&app);
                }

                was_pressed = pressed;
                thread::sleep(POLL_INTERVAL);
            }
        })
        .map(|_| ())
        .map_err(|error| format!("无法启动快捷键监听线程：{error}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn polling_interval_is_short_enough_for_a_key_press() {
        assert!(super::POLL_INTERVAL.as_millis() <= 10);
    }
}
