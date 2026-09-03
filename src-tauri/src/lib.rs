mod config;
mod hotkey;
mod message_file;
mod platform;
mod runner;
mod state;

use state::{RuntimeState, Snapshot};
use std::{fs, path::PathBuf, process::Command};
use tauri::{AppHandle, Manager};

pub fn portable_dir() -> Result<PathBuf, String> {
    #[cfg(debug_assertions)]
    {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("src-tauri has a parent")
            .to_path_buf());
    }

    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .map_err(|error| format!("无法定位 EXE：{error}"))?
            .parent()
            .map(PathBuf::from)
            .ok_or_else(|| "无法定位 EXE 所在目录".to_string())
    }
}

fn ensure_portable_files() -> Result<(), String> {
    let dir = portable_dir()?;
    let messages = dir.join("messages.txt");
    let config = dir.join("config.toml");
    if !messages.exists() {
        fs::write(&messages, message_file::DEFAULT_MESSAGES)
            .map_err(|error| format!("无法创建 messages.txt：{error}"))?;
    }
    if !config.exists() {
        fs::write(&config, config::DEFAULT_CONFIG)
            .map_err(|error| format!("无法创建 config.toml：{error}"))?;
    }
    Ok(())
}

fn load_snapshot() -> Result<Snapshot, String> {
    ensure_portable_files()?;
    let dir = portable_dir()?;
    let message_path = dir.join("messages.txt");
    let app_config = config::load(&dir.join("config.toml"))?;
    let messages = message_file::load(&message_path)?;
    Ok(Snapshot::idle(
        message_path.display().to_string(),
        app_config.interval_ms,
        messages.len(),
    ))
}

#[tauri::command]
fn get_snapshot(app: AppHandle) -> Result<Snapshot, String> {
    let state = app.state::<RuntimeState>();
    if state.running.load(std::sync::atomic::Ordering::Acquire) {
        Ok(state.snapshot())
    } else {
        let snapshot = load_snapshot()?;
        state.publish(&app, snapshot.clone());
        Ok(snapshot)
    }
}

#[tauri::command]
fn toggle_run(app: AppHandle) {
    runner::toggle(&app);
}

#[tauri::command]
fn open_messages_file() -> Result<(), String> {
    ensure_portable_files()?;
    let path = portable_dir()?.join("messages.txt");
    Command::new("notepad.exe")
        .arg(path)
        .spawn()
        .map_err(|error| format!("无法打开记事本：{error}"))?;
    Ok(())
}

#[tauri::command]
fn open_app_folder() -> Result<(), String> {
    let path = portable_dir()?;
    Command::new("explorer.exe")
        .arg(path)
        .spawn()
        .map_err(|error| format!("无法打开程序目录：{error}"))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial = load_snapshot().unwrap_or_else(|error| Snapshot {
        phase: "error".into(),
        detail: error,
        current: 0,
        total: 0,
        message_file: "messages.txt".into(),
        interval_ms: 1_000,
        hotkey: "Ctrl+F3".into(),
    });
    tauri::Builder::default()
        .manage(RuntimeState::new(initial))
        .setup(|app| {
            hotkey::start(app.handle().clone())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            toggle_run,
            open_messages_file,
            open_app_folder
        ])
        .run(tauri::generate_context!())
        .expect("GkSay 启动失败");
}
