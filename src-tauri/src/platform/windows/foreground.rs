use std::path::Path;
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

pub fn process_name() -> Result<String, String> {
    let window = unsafe { GetForegroundWindow() };
    if window.is_null() {
        return Err("当前没有前台窗口".to_string());
    }

    let mut process_id = 0;
    unsafe { GetWindowThreadProcessId(window, &mut process_id) };
    if process_id == 0 {
        return Err("无法获得前台进程 ID".to_string());
    }

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    if process.is_null() {
        return Err("无法读取前台进程信息（可能是权限等级不同）".to_string());
    }

    let mut buffer = vec![0_u16; 32_768];
    let mut length = buffer.len() as u32;
    let result =
        unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) };
    unsafe { CloseHandle(process) };

    if result == 0 {
        return Err("无法读取前台进程路径".to_string());
    }

    let full_path = String::from_utf16_lossy(&buffer[..length as usize]);
    Ok(Path::new(&full_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&full_path)
        .to_string())
}

pub fn is_allowed(allowed: &[String]) -> Result<(), String> {
    let current = process_name()?;
    if allowed
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&current))
    {
        Ok(())
    } else {
        Err(format!("当前前台程序是 {current}，不是英雄联盟"))
    }
}
