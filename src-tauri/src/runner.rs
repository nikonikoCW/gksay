use crate::{
    config, message_file,
    platform::windows::{foreground, input},
    state::{RuntimeState, Snapshot},
};
use arboard::Clipboard;
use std::{
    path::PathBuf,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

pub fn toggle(app: &AppHandle) {
    let state = app.state::<RuntimeState>();

    if state.running.load(Ordering::Acquire) {
        state.request_stop();
        let mut snapshot = state.snapshot();
        snapshot.phase = "stopping".into();
        snapshot.detail = "正在停止；当前消息完成后结束".into();
        state.publish(app, snapshot);
        return;
    }

    if state
        .running
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    state.cancel.store(false, Ordering::Release);

    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = run_once(&app).await {
            let state = app.state::<RuntimeState>();
            let mut snapshot = state.snapshot();
            snapshot.phase = "error".into();
            snapshot.detail = error;
            state.publish(&app, snapshot);
        }
        let state = app.state::<RuntimeState>();
        state.running.store(false, Ordering::Release);
        state.cancel.store(false, Ordering::Release);
    });
}

async fn run_once(app: &AppHandle) -> Result<(), String> {
    let app_dir = crate::portable_dir()?;
    let message_path = app_dir.join("messages.txt");
    let config_path = app_dir.join("config.toml");
    let config = config::load(&config_path)?;
    let messages = message_file::load(&message_path)?;
    if messages.is_empty() {
        return Err("messages.txt 没有可发送的内容".into());
    }

    let state = app.state::<RuntimeState>();
    let total = messages.len();
    state.publish(
        app,
        Snapshot {
            phase: "running".into(),
            detail: "发送任务已启动".into(),
            current: 0,
            total,
            message_file: message_path.display().to_string(),
            interval_ms: config.interval_ms,
            hotkey: "Ctrl+F3".into(),
        },
    );

    for (index, message) in messages.iter().enumerate() {
        if state.cancel.load(Ordering::Acquire) {
            return finish_stopped(app, &message_path, config.interval_ms, index, total);
        }

        if config.require_lol_foreground {
            foreground::is_allowed(&config.allowed_processes)?;
        }

        let started = Instant::now();
        state.publish(
            app,
            Snapshot {
                phase: "running".into(),
                detail: format!("正在发送第 {} 条", index + 1),
                current: index,
                total,
                message_file: message_path.display().to_string(),
                interval_ms: config.interval_ms,
                hotkey: "Ctrl+F3".into(),
            },
        );

        send_message(
            message,
            config.open_chat_delay_ms,
            config.paste_delay_ms,
            config.restore_clipboard,
        )
        .await?;

        state.publish(
            app,
            Snapshot {
                phase: "running".into(),
                detail: format!("已发送第 {} 条", index + 1),
                current: index + 1,
                total,
                message_file: message_path.display().to_string(),
                interval_ms: config.interval_ms,
                hotkey: "Ctrl+F3".into(),
            },
        );

        if index + 1 < total {
            let target = Duration::from_millis(config.interval_ms);
            if let Some(remaining) = target.checked_sub(started.elapsed()) {
                interruptible_sleep(remaining, &state).await;
            }
        }
    }

    state.publish(
        app,
        Snapshot {
            phase: "completed".into(),
            detail: format!("发送完成，共 {total} 条"),
            current: total,
            total,
            message_file: message_path.display().to_string(),
            interval_ms: config.interval_ms,
            hotkey: "Ctrl+F3".into(),
        },
    );
    Ok(())
}

async fn send_message(
    message: &str,
    open_chat_delay_ms: u64,
    paste_delay_ms: u64,
    restore_clipboard: bool,
) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|error| format!("无法打开剪贴板：{error}"))?;
    let previous = restore_clipboard
        .then(|| clipboard.get_text().ok())
        .flatten();

    input::press_enter()?;
    tokio_sleep(open_chat_delay_ms).await;

    clipboard
        .set_text(message.to_string())
        .map_err(|error| format!("无法写入剪贴板：{error}"))?;
    input::press_ctrl_v()?;
    tokio_sleep(paste_delay_ms).await;
    input::press_enter()?;

    if restore_clipboard && clipboard.get_text().ok().as_deref() == Some(message) {
        if let Some(previous) = previous {
            let _ = clipboard.set_text(previous);
        }
    }
    Ok(())
}

async fn tokio_sleep(milliseconds: u64) {
    tokio::time::sleep(Duration::from_millis(milliseconds)).await;
}

async fn interruptible_sleep(duration: Duration, state: &RuntimeState) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline && !state.cancel.load(Ordering::Acquire) {
        let remaining = deadline.saturating_duration_since(Instant::now());
        tokio_sleep(remaining.min(Duration::from_millis(25)).as_millis() as u64).await;
    }
}

fn finish_stopped(
    app: &AppHandle,
    message_path: &PathBuf,
    interval_ms: u64,
    current: usize,
    total: usize,
) -> Result<(), String> {
    app.state::<RuntimeState>().publish(
        app,
        Snapshot {
            phase: "idle".into(),
            detail: "发送已停止".into(),
            current,
            total,
            message_file: message_path.display().to_string(),
            interval_ms,
            hotkey: "Ctrl+F3".into(),
        },
    );
    Ok(())
}
