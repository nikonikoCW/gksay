import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./style.css";

type Phase = "idle" | "running" | "stopping" | "completed" | "error";

interface Snapshot {
  phase: Phase;
  detail: string;
  current: number;
  total: number;
  messageFile: string;
  intervalMs: number;
  hotkey: string;
}

const app = document.querySelector<HTMLElement>("#app")!;

app.innerHTML = `
  <section class="shell">
    <header>
      <div class="brand">GK</div>
      <div>
        <h1>GkSay</h1>
        <p>从 EXE 同目录的 messages.txt 依次发送</p>
      </div>
      <span id="badge" class="badge idle">等待中</span>
    </header>

    <div class="status-card">
      <div class="status-topline">
        <span id="detail">正在初始化…</span>
        <strong id="progress">0 / 0</strong>
      </div>
      <div class="progress-track"><div id="progress-bar"></div></div>
    </div>

    <dl class="facts">
      <div><dt>快捷键</dt><dd id="hotkey">Ctrl+F3</dd></div>
      <div><dt>发送间隔</dt><dd id="interval">1000 ms</dd></div>
      <div><dt>消息数量</dt><dd id="count">0 条</dd></div>
    </dl>

    <div class="path-block">
      <span>消息文件</span>
      <code id="path">messages.txt</code>
    </div>

    <div class="actions">
      <button id="toggle" class="primary">开始发送</button>
      <button id="refresh">重新读取</button>
      <button id="open">打开 messages.txt</button>
      <button id="folder">打开程序目录</button>
    </div>

    <p class="hint">进入游戏后按 Ctrl+F3 开始；发送过程中再次按下可停止。</p>
  </section>
`;

const el = <T extends HTMLElement>(id: string) => document.querySelector<T>(`#${id}`)!;
const toggleButton = el<HTMLButtonElement>("toggle");

const labels: Record<Phase, string> = {
  idle: "等待中",
  running: "发送中",
  stopping: "停止中",
  completed: "已完成",
  error: "出错了",
};

function render(snapshot: Snapshot) {
  const badge = el("badge");
  badge.textContent = labels[snapshot.phase];
  badge.className = `badge ${snapshot.phase}`;
  el("detail").textContent = snapshot.detail;
  el("progress").textContent = `${snapshot.current} / ${snapshot.total}`;
  el("progress-bar").style.width = snapshot.total
    ? `${Math.min(100, (snapshot.current / snapshot.total) * 100)}%`
    : "0%";
  el("hotkey").textContent = snapshot.hotkey;
  el("interval").textContent = `${snapshot.intervalMs} ms`;
  el("count").textContent = `${snapshot.total} 条`;
  el("path").textContent = snapshot.messageFile;

  const active = snapshot.phase === "running" || snapshot.phase === "stopping";
  toggleButton.textContent = active ? "停止发送" : "开始发送";
  toggleButton.classList.toggle("danger", active);
}

async function refresh() {
  try {
    render(await invoke<Snapshot>("get_snapshot"));
  } catch (error) {
    render({
      phase: "error",
      detail: String(error),
      current: 0,
      total: 0,
      messageFile: "messages.txt",
      intervalMs: 1000,
      hotkey: "Ctrl+F3",
    });
  }
}

toggleButton.addEventListener("click", () => invoke("toggle_run"));
el("refresh").addEventListener("click", refresh);
el("open").addEventListener("click", () => invoke("open_messages_file"));
el("folder").addEventListener("click", () => invoke("open_app_folder"));

await listen<Snapshot>("gksay-status", ({ payload }) => render(payload));
await refresh();
