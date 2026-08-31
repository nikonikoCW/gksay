use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub phase: String,
    pub detail: String,
    pub current: usize,
    pub total: usize,
    pub message_file: String,
    pub interval_ms: u64,
    pub hotkey: String,
}

impl Snapshot {
    pub fn idle(message_file: String, interval_ms: u64, total: usize) -> Self {
        Self {
            phase: "idle".into(),
            detail: "等待 Ctrl+F3".into(),
            current: 0,
            total,
            message_file,
            interval_ms,
            hotkey: "Ctrl+F3".into(),
        }
    }
}

pub struct RuntimeState {
    pub running: AtomicBool,
    pub cancel: AtomicBool,
    snapshot: Mutex<Snapshot>,
}

impl RuntimeState {
    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            running: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            snapshot: Mutex::new(snapshot),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot
            .lock()
            .expect("snapshot mutex poisoned")
            .clone()
    }

    pub fn publish(&self, app: &AppHandle, snapshot: Snapshot) {
        *self.snapshot.lock().expect("snapshot mutex poisoned") = snapshot.clone();
        let _ = app.emit("gksay-status", snapshot);
    }

    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::Release);
    }
}
