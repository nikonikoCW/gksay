<p align="center">
  <img src="assets/app-icon.png" width="128" alt="GkSay 图标">
</p>

<h1 align="center">GkSay</h1>

<p align="center">
  一个使用 Rust + Tauri 2 编写的 Windows 便携式文本队列发送工具。
</p>

GkSay 从程序同目录的 `messages.txt` 读取消息。按下全局快捷键后，程序会将每一行依次
通过 `Enter → Ctrl+V → Enter` 发送到当前前台窗口。它不注入进程，也不读取或修改其他
程序的内存。

## 功能

- `Ctrl+F3` 底层按键状态检测开始或停止发送，兼容常见全屏窗口
- 每次开始时重新读取 `messages.txt` 和 `config.toml`，修改后无需重启
- 支持 UTF-8 中文文本，一行一条消息
- 自动忽略空行以及以 `#` 开头的注释
- 可配置消息间隔、聊天框等待时间和粘贴等待时间
- 可在每条消息发送后恢复原剪贴板文本
- 可选前台进程白名单，默认关闭，可用于减少误发
- 纯便携目录运行，不依赖安装程序

## 快速使用

推荐从源码构建，或使用项目维护者提供的便携包。便携目录结构如下：

```text
GkSay/
├─ GkSay.exe
├─ messages.txt
├─ config.toml
└─ 使用说明.txt
```

1. 使用记事本或其他文本编辑器打开 `messages.txt`。
2. 每行填写一句需要发送的文字并保存为 UTF-8。
3. 双击 `GkSay.exe`。
4. 切换到需要接收输入的窗口。
5. 按 `Ctrl+F3` 开始发送；再次按下可停止。

> GkSay 会向当前前台窗口发送真实的系统键盘输入。启动前请确认焦点位置，避免把内容
> 误发到聊天软件、终端、表单或其他窗口。

## messages.txt 格式

```text
# 这是注释，不会发送
大家好

准备打小龙
先别开团
```

上面的文件会依次发送三条消息。空行及注释不会计入消息数量。

## 配置

默认 `config.toml`：

```toml
interval_ms = 1000
open_chat_delay_ms = 120
paste_delay_ms = 100
require_lol_foreground = false
restore_clipboard = true

allowed_processes = ["League of Legends.exe"]
```

| 字段 | 说明 |
| --- | --- |
| `interval_ms` | 两条消息开始发送之间的间隔，程序最低限制为 300 ms |
| `open_chat_delay_ms` | 第一次按 Enter 后，等待输入框打开的时间 |
| `paste_delay_ms` | 粘贴文字后，等待最终 Enter 的时间 |
| `restore_clipboard` | 发送后是否恢复此前的文本剪贴板 |
| `require_lol_foreground` | 是否启用前台进程白名单；默认 `false` |
| `allowed_processes` | 开启白名单后允许接收输入的进程名列表 |

虽然配置字段沿用了最初面向英雄联盟使用场景的名字，但默认不会限制目标程序。设为
`true` 后，只有 `allowed_processes` 中列出的前台进程才会接收输入。

## 从源码开发

### 环境要求

- Windows 10/11
- Rust stable（MSVC toolchain）
- Node.js 20+
- pnpm
- Microsoft Edge WebView2 Runtime

安装依赖并启动开发模式：

```powershell
pnpm install
pnpm tauri dev
```

执行检查：

```powershell
pnpm check
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo check
```

## 构建便携版

```powershell
pnpm build
cd src-tauri
cargo build --release --features tauri/custom-protocol
cd ..
.\package-portable.ps1
```

输出位于 `portable/GkSay/`。`node_modules`、`dist`、Rust `target`、EXE 和 ZIP 等构建产物
不会提交到源码仓库。

## 项目结构

```text
src/                          Tauri WebView 前端
src-tauri/src/config.rs       配置读取与默认值
src-tauri/src/message_file.rs 消息文件解析
src-tauri/src/hotkey.rs       全屏兼容的 Win32 快捷键检测
src-tauri/src/runner.rs       发送队列和停止控制
src-tauri/src/platform/windows/input.rs       Win32 SendInput
src-tauri/src/platform/windows/foreground.rs  前台进程检测
package-portable.ps1          便携目录打包脚本
```

## 使用提醒

某些软件或游戏可能阻止模拟输入。如果目标程序以管理员身份运行，GkSay 通常也需要使用
相同权限等级。使用自动发送功能时，请遵守目标平台的服务条款和聊天规则，避免刷屏或
干扰他人。
