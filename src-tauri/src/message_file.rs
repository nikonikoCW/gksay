use std::{fs, path::Path};

pub const DEFAULT_MESSAGES: &str =
    "# 每行一条消息；空行和以 # 开头的行不会发送\n大家好\n准备打小龙\n先别开团\n";

pub fn load(path: &Path) -> Result<Vec<String>, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("无法读取 messages.txt：{error}"))?;

    Ok(content
        .lines()
        .map(|line| line.trim().trim_start_matches('\u{feff}'))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_blank_lines_comments_and_bom() {
        let dir = std::env::temp_dir();
        let path = dir.join("gksay-message-file-test.txt");
        fs::write(&path, "\u{feff}第一句\n\n# 注释\n  第二句  \n").unwrap();
        assert_eq!(load(&path).unwrap(), vec!["第一句", "第二句"]);
        let _ = fs::remove_file(path);
    }
}
