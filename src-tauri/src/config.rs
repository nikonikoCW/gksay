use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub interval_ms: u64,
    pub open_chat_delay_ms: u64,
    pub paste_delay_ms: u64,
    pub require_lol_foreground: bool,
    pub restore_clipboard: bool,
    pub allowed_processes: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interval_ms: 1_000,
            open_chat_delay_ms: 120,
            paste_delay_ms: 100,
            require_lol_foreground: false,
            restore_clipboard: true,
            allowed_processes: vec!["League of Legends.exe".to_string()],
        }
    }
}

pub const DEFAULT_CONFIG: &str = r#"# GkSay 配置文件（保存后，下次按 Ctrl+F3 时生效）
interval_ms = 1000
open_chat_delay_ms = 120
paste_delay_ms = 100
require_lol_foreground = false
restore_clipboard = true

# 允许接收消息的前台进程名，可按实际情况增加
allowed_processes = ["League of Legends.exe"]
"#;

pub fn load(path: &Path) -> Result<AppConfig, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("无法读取 config.toml：{error}"))?;
    let mut config: AppConfig =
        toml::from_str(&text).map_err(|error| format!("config.toml 格式错误：{error}"))?;
    config.interval_ms = config.interval_ms.max(300);
    config.open_chat_delay_ms = config.open_chat_delay_ms.clamp(30, 2_000);
    config.paste_delay_ms = config.paste_delay_ms.clamp(30, 2_000);
    if config.allowed_processes.is_empty() {
        config.allowed_processes = AppConfig::default().allowed_processes;
    }
    Ok(config)
}
