# WezTeam 开发计划

> Fork WezTerm，实现 TeamShell 通讯协议

## 1. 项目概述

### 1.1 目标

在 WezTerm 开源终端模拟器的基础上，实现 TeamShell 多 Agent 通讯协议，使 WezTeam 成为一个支持多标签页、Agent 间消息传递的原生终端。

### 1.2 核心差异：WezTerm vs WezTeam

| 维度 | WezTerm | WezTeam |
|------|---------|---------|
| 定位 | 通用终端模拟器 | 多 Agent 协作终端 |
| IPC | 二进制 PDU over Unix Socket | **JSON over Named Pipe / Unix Socket** |
| Tab 管理 | 用户手动操作 | **tsh CLI 远程管理** |
| 跨 Tab 通信 | 无 | **send/see/list 命令** |
| 环境变量 | 无 | **TEAMSH_TAB_ID, TEAMSH_NAME** |
| 输出格式 | GUI 渲染 | **tsh 输出优化为 LLM 友好格式** |

### 1.3 技术栈

- **语言**: Rust
- **基础**: WezTerm (MIT License)
- **IPC**: tokio (Named Pipe on Windows, Unix Socket on Linux/macOS)
- **协议**: newline-delimited JSON
- **PTY**: portable-pty (已有，Windows ConPTY + Unix forkpty)

---

## 2. 架构设计

### 2.1 系统架构

```
┌─────────────────────────────────────────────────────┐
│                    WezTeam GUI                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │  Tab 1  │  │  Tab 2  │  │  Tab 3  │  ...        │
│  │ (Alice) │  │  (Bob)  │  │(Charlie)│             │
│  └────┬────┘  └────┬────┘  └────┬────┘             │
│       │            │            │                    │
│  ┌────┴────────────┴────────────┴────┐              │
│  │           Mux (多路复用器)          │              │
│  │  ┌──────────────────────────────┐ │              │
│  │  │    TeamShell Protocol Layer   │ │              │
│  │  │  (Named Pipe + JSON Handler)  │ │              │
│  │  └──────────────────────────────┘ │              │
│  └───────────────────────────────────┘              │
└─────────────────────────────────────────────────────┘
         ▲                    ▲
         │ Named Pipe/Socket  │ Named Pipe/Socket
         │                    │
    ┌────┴────┐          ┌────┴────┐
    │  tsh 1  │          │  tsh 2  │  ...
    │ (Alice) │          │  (Bob)  │
    └─────────┘          └─────────┘
```

### 2.2 协议层位置

TeamShell 协议层实现为 `wezterm-mux-server-impl` 的一个新模块，绕过 WezTerm 原有的二进制 PDU 协议：

```
wezterm-mux-server-impl/src/
├── lib.rs                    (已有 - 领域配置)
├── dispatch.rs               (已有 - PDU 分发)
├── local.rs                  (已有 - Unix Socket 监听)
├── sessionhandler.rs         (已有 - 会话处理)
├── pki.rs                    (已有 - TLS)
└── teamshell/                (新增 - TeamShell 协议)
    ├── mod.rs                (模块入口)
    ├── protocol.rs           (IpcRequest/IpcResponse 定义)
    ├── server.rs             (Named Pipe / Unix Socket 服务端)
    ├── handler.rs            (命令处理器 - 调用 Mux API)
    └── output.rs             (LLM 友好输出格式化)
```

### 2.3 数据流

**Agent 发送消息 (tsh send 2 "hello"):**

```
tsh CLI
  → 生成 JSON: {"cmd":"send","tab_index":2,"message":"hello","from_tab_id":1}\n
  → 写入 Named Pipe
  → WezTeam server.rs 读取
  → handler.rs 解析 JSON
  → 调用 mux.get_pane(pane_id)
  → pane.send_paste("[TeamShell 消息] from Alice: hello\n")
  → 目标 Tab 的终端显示消息
```

**Agent 读取屏幕 (tsh see 2):**

```
tsh CLI
  → 生成 JSON: {"cmd":"see","tab_index":2,"line_count":50}\n
  → 写入 Named Pipe
  → WezTeam server.rs 读取
  → handler.rs 解析 JSON
  → 调用 mux.get_pane(pane_id)
  → pane.get_lines(range) 获取终端内容
  → output.rs 格式化为纯文本
  → 写回 Named Pipe
  → tsh CLI 输出到 stdout
```

---

## 3. 协议定义

### 3.1 IPC 协议 (内部 - JSON)

tsh CLI 与 WezTeam 后端之间的通信格式。保持 JSON，因为这是内部实现细节。

```rust
// teamshell/protocol.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcRequest {
    #[serde(rename = "send")]
    Send {
        tab_index: usize,
        message: String,
        from_tab_id: Option<usize>,
    },
    #[serde(rename = "send_raw")]
    SendRaw { tab_index: usize, data: String },
    #[serde(rename = "see")]
    See { tab_index: usize, line_count: usize },
    #[serde(rename = "list")]
    List,
    #[serde(rename = "open")]
    Open {
        name: String,
        command: Option<String>,
        args: Option<Vec<String>>,
        cwd: Option<String>,
        env: Option<Vec<(String, String)>>,
    },
    #[serde(rename = "close")]
    Close { tab_index: usize },
    #[serde(rename = "name")]
    Name { tab_index: usize, new_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<TabInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub index: usize,
    pub name: String,
}
```

### 3.2 tsh 输出格式 (外部 - LLM 友好)

tsh CLI 输出到 stdout 的格式，面向大模型优化。

#### tsh list

```bash
# 当前 (JSON)
{"ok":true,"tabs":[{"index":1,"name":"Alice"},{"index":2,"name":"Bob"}]}

# 优化后 (纯文本)
1 Alice
2 Bob
```

#### tsh see <id> [lines]

```bash
# 当前 (JSON)
{"ok":true,"text":"$ cargo build\nCompiling...\nDone."}

# 优化后 (纯文本，直接输出内容)
$ cargo build
Compiling teamshell...
Done.
```

#### tsh send <id> "msg"

```bash
# 当前 (JSON)
{"ok":true,"status":"delivered","target":2}

# 优化后 (简洁确认)
OK → 2
```

#### tsh open "name" cmd args

```bash
# 当前 (JSON)
{"ok":true,"status":"created","tab":3}

# 优化后
Created tab 3 (Charlie)
```

#### tsh close <id>

```bash
# 当前 (JSON)
{"ok":true,"status":"closed","target":3}

# 优化后
Closed tab 3
```

#### tsh name <id> "new_name"

```bash
# 当前 (JSON)
{"ok":true,"status":"renamed","target":2}

# 优化后
Tab 2 renamed to "Bob"
```

#### 错误情况

```bash
# 所有错误统一格式
Error: tab 99 not found
Error: pty create failed: permission denied
Error: connection refused (is WezTeam running?)
```

---

## 4. 实施阶段

### Phase 1: 基础设施 (2-3天)

**目标**: Fork WezTerm，确保编译通过，建立开发环境。

#### 1.1 项目初始化

```bash
# 已完成: research/wezterm → wezteam/
cd wezteam
git init
git remote add upstream https://github.com/wez/wezterm.git
git commit -m "fork: initial WezTerm codebase"
```

#### 1.2 编译验证

```bash
# Windows
cargo build --release

# 验证基础功能
cargo run --release
```

#### 1.3 清理无用功能

在 `Cargo.toml` 中禁用不需要的 features：
- SSH domains
- TLS support
- Serial ports
- tmux integration (保留代码，但不编译)

**产出**: 可编译运行的 WezTeam 基础版本。

---

### Phase 2: TeamShell 协议 Crate (3-5天)

**目标**: 实现 TeamShell IPC 协议层。

#### 2.1 创建 teamshell 模块

在 `wezterm-mux-server-impl/src/` 下创建 `teamshell/` 目录：

```
wezterm-mux-server-impl/src/teamshell/
├── mod.rs          # 模块入口
├── protocol.rs     # IpcRequest/IpcResponse
├── server.rs       # Named Pipe / Unix Socket 服务
├── handler.rs      # 命令处理
└── output.rs       # 输出格式化
```

#### 2.2 实现 protocol.rs

从 `team2/src/backend/src/protocol.rs` 迁移协议定义。

#### 2.3 实现 server.rs

**Windows (Named Pipe):**

```rust
use tokio::net::windows::named_pipe::ServerOptions;

const PIPE_NAME: &str = r"\\.\pipe\TeamShell";

pub async fn start_server(mux: Arc<Mux>) {
    let server = ServerOptions::new()
        .first_pipe_instance(true)
        .create(PIPE_NAME)
        .expect("Failed to create Named Pipe");

    loop {
        server.connect().await.expect("Failed to connect");
        let client = server;
        let mux = mux.clone();

        tokio::spawn(async move {
            handle_client(client, mux).await;
        });

        // 创建下一个 pipe 实例
        server = ServerOptions::new()
            .create(PIPE_NAME)
            .expect("Failed to create next pipe instance");
    }
}
```

**Linux/macOS (Unix Socket):**

```rust
use tokio::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/TeamShell.sock";

pub async fn start_server(mux: Arc<Mux>) {
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to bind");

    loop {
        let (stream, _) = listener.accept().await.expect("Failed to accept");
        let mux = mux.clone();

        tokio::spawn(async move {
            handle_client(stream, mux).await;
        });
    }
}
```

#### 2.4 实现 handler.rs

核心命令处理器，调用 Mux API：

```rust
use mux::Mux;

pub fn handle_command(mux: &Mux, request: &IpcRequest) -> IpcResponse {
    match request {
        IpcRequest::List => {
            let tabs = list_tabs(mux);
            IpcResponse {
                ok: true,
                tabs: Some(tabs),
                ..Default::default()
            }
        }

        IpcRequest::Send { tab_index, message, from_tab_id } => {
            let pane_id = tab_index - 1; // 1-based → 0-based
            if let Some(pane) = mux.get_pane(pane_id) {
                let wrapped = wrap_agent_message(*from_tab_id, message);
                let _ = pane.send_paste(&wrapped);
                IpcResponse {
                    ok: true,
                    status: Some("delivered".into()),
                    target: Some(*tab_index),
                    ..Default::default()
                }
            } else {
                error_response("tab_not_found")
            }
        }

        IpcRequest::SendRaw { tab_index, data } => {
            let pane_id = tab_index - 1;
            if let Some(pane) = mux.get_pane(pane_id) {
                let _ = pane.send_paste(data);
                IpcResponse {
                    ok: true,
                    status: Some("sent".into()),
                    target: Some(*tab_index),
                    ..Default::default()
                }
            } else {
                error_response("tab_not_found")
            }
        }

        IpcRequest::See { tab_index, line_count } => {
            let pane_id = tab_index - 1;
            if let Some(pane) = mux.get_pane(pane_id) {
                let lines = read_pane_output(&*pane, *line_count);
                IpcResponse {
                    ok: true,
                    lines: Some(lines),
                    ..Default::default()
                }
            } else {
                error_response("tab_not_found")
            }
        }

        IpcRequest::Open { name, command, args, cwd, env } => {
            match create_tab(mux, name, command.as_deref(), args.as_deref(), cwd.as_deref(), env.as_deref()) {
                Ok(tab_id) => IpcResponse {
                    ok: true,
                    status: Some("created".into()),
                    tab: Some(tab_id + 1), // 0-based → 1-based
                    ..Default::default()
                },
                Err(e) => error_response(&format!("pty_error: {}", e)),
            }
        }

        IpcRequest::Close { tab_index } => {
            let pane_id = tab_index - 1;
            if let Some(pane) = mux.get_pane(pane_id) {
                pane.kill();
                IpcResponse {
                    ok: true,
                    status: Some("closed".into()),
                    target: Some(*tab_index),
                    ..Default::default()
                }
            } else {
                error_response("tab_not_found")
            }
        }

        IpcRequest::Name { tab_index, new_name } => {
            // 实现 tab 改名
            IpcResponse {
                ok: true,
                status: Some("renamed".into()),
                target: Some(*tab_index),
                ..Default::default()
            }
        }
    }
}
```

#### 2.5 实现 output.rs

LLM 友好输出格式化：

```rust
pub fn format_list_response(tabs: &[TabInfo]) -> String {
    tabs.iter()
        .map(|t| format!("{} {}", t.index, t.name))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn format_send_response(tab_index: usize) -> String {
    format!("OK → {}", tab_index)
}

pub fn format_see_response(text: &str) -> String {
    text.to_string()
}

pub fn format_open_response(tab_id: usize, name: &str) -> String {
    format!("Created tab {} ({})", tab_id, name)
}

pub fn format_close_response(tab_index: usize) -> String {
    format!("Closed tab {}", tab_index)
}

pub fn format_name_response(tab_index: usize, new_name: &str) -> String {
    format!("Tab {} renamed to \"{}\"", tab_index, new_name)
}

pub fn format_error(msg: &str) -> String {
    format!("Error: {}", msg)
}
```

**产出**: TeamShell 协议层实现完成。

---

### Phase 3: Tab 管理与环境变量 (2-3天)

**目标**: 实现 Tab 的完整生命周期管理。

#### 3.1 Tab 创建 (open)

在 handler.rs 中实现 `create_tab`:

```rust
fn create_tab(
    mux: &Mux,
    name: &str,
    command: Option<&str>,
    args: Option<&[String]>,
    cwd: Option<&str>,
    env: Option<&[(String, String)]>,
) -> Result<usize, anyhow::Error> {
    let domain = mux.get_default_domain().unwrap();

    // 构建命令
    let mut cmd = match command {
        Some(c) => CommandBuilder::new(c),
        None => CommandBuilder::new(detect_shell()),
    };

    // 添加参数
    if let Some(args) = args {
        for arg in args {
            cmd.arg(arg);
        }
    }

    // 设置工作目录
    if let Some(cwd) = cwd {
        cmd.cwd(cwd);
    }

    // 注入环境变量
    for (k, v) in env.unwrap_or_default() {
        cmd.env(k, v);
    }

    // 注入 TEAMSH_TAB_ID 和 TEAMSH_NAME (临时，实际 ID 在创建后分配)
    cmd.env("TEAMSH_TAB_NAME", name);

    // 创建 Tab
    let tab = domain.spawn_tab(None, None, &cmd, None)?;
    let tab_id = tab.tab_id();

    // 更新环境变量 (需要重新设置实际的 TEAMSH_TAB_ID)
    // 这需要在 PTY 创建后注入

    Ok(tab_id)
}
```

#### 3.2 Tab 改名 (name)

```rust
fn rename_tab(mux: &Mux, tab_index: usize, new_name: &str) -> Result<(), anyhow::Error> {
    // WezTerm 的 Tab 有 set_title 方法
    if let Some(tab) = mux.get_tab(tab_index - 1) {
        tab.set_title(new_name);
        Ok(())
    } else {
        Err(anyhow!("tab_not_found"))
    }
}
```

#### 3.3 环境变量注入

在 `localpane.rs` 的 PTY 创建过程中注入环境变量：

```rust
// 在 LocalPane::new 或类似的创建函数中
fn inject_teamshell_env(
    cmd: &mut CommandBuilder,
    tab_id: usize,
    tab_name: &str,
) {
    cmd.env("TEAMSH_TAB_ID", tab_id.to_string());
    cmd.env("TEAMSH_NAME", tab_name);
}
```

#### 3.4 Tab 列表 (list)

```rust
fn list_tabs(mux: &Mux) -> Vec<TabInfo> {
    let mut tabs = Vec::new();

    // 遍历所有 window 和 tab
    for window_id in mux.iter_windows() {
        if let Some(window) = mux.get_window(window_id) {
            for tab_id in window.iter() {
                if let Some(tab) = mux.get_tab(tab_id) {
                    let name = tab.get_title();
                    let pane_id = tab_id + 1; // 1-based
                    tabs.push(TabInfo {
                        index: pane_id,
                        name,
                    });
                }
            }
        }
    }

    tabs
}
```

**产出**: Tab 完整生命周期管理实现。

---

### Phase 4: tsh CLI 适配 (1-2天)

**目标**: 确保 tsh CLI 与 WezTeam 后端正常通信。

#### 4.1 IPC 客户端适配

tsh CLI 的 IPC 客户端 (`tsh/src/ipc.rs`) 已经支持 Named Pipe，只需确认：

1. Pipe 名称一致: `\\.\pipe\TeamShell`
2. JSON 协议格式一致
3. 重试逻辑正常

#### 4.2 输出格式调整

在 `tsh/src/commands.rs` 中修改输出格式：

```rust
// list 命令
pub fn cmd_list() {
    let request = Request::List;
    match ipc::send_request(&request) {
        Ok(resp) => {
            if let Some(tabs) = resp.tabs {
                for tab in tabs {
                    println!("{} {}", tab.index, tab.name);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

// see 命令
pub fn cmd_see(id: usize, lines: usize) {
    let request = Request::See { tab_index: id, line_count: lines };
    match ipc::send_request(&request) {
        Ok(resp) => {
            if let Some(text) = resp.text {
                print!("{}", text);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

// send 命令
pub fn cmd_send(id: usize, message: Option<&str>, auto_enter: bool, key: Option<&str>) {
    // ... 发送逻辑 ...
    // 输出: OK → {id}
    println!("OK → {}", id);
}
```

#### 4.3 测试验证

```bash
# 启动 WezTeam
cargo run --release

# 在另一个终端测试 tsh
tsh list                    # 应显示: 1 bash
tsh open "Alice" bash       # 应显示: Created tab 2 (Alice)
tsh list                    # 应显示: 1 bash\n2 Alice
tsh send 2 "hello"          # 应显示: OK → 2
tsh see 2                   # 应显示 Alice 终端的内容
tsh close 2                 # 应显示: Closed tab 2
```

**产出**: tsh CLI 与 WezTeam 完全兼容。

---

### Phase 5: 完善与测试 (2-3天)

**目标**: 完善功能，修复问题，添加测试。

#### 5.1 空闲检测

在 `handler.rs` 中添加空闲检测逻辑：

```rust
use std::time::{Duration, Instant};

struct TabState {
    last_input: Instant,
    last_output: Instant,
}

fn is_idle(tab_state: &TabState, timeout: Duration) -> bool {
    let now = Instant::now();
    now.duration_since(tab_state.last_input) > timeout
        && now.duration_since(tab_state.last_output) > timeout
}
```

#### 5.2 消息包装

Agent 间消息自动添加 `[TeamShell 消息]` 前缀：

```rust
fn wrap_agent_message(from_tab_id: Option<usize>, message: &str) -> String {
    match from_tab_id {
        Some(id) if id > 0 => {
            // Agent 间消息
            let sender_name = get_tab_name(id);
            format!("[TeamShell 消息] from {}: {}\n", sender_name, message)
        }
        _ => {
            // 用户消息或外部 CLI 消息
            format!("{}\n", message)
        }
    }
}
```

#### 5.3 错误处理

统一错误格式：

```rust
fn error_response(msg: &str) -> IpcResponse {
    IpcResponse {
        ok: false,
        error: Some(msg.to_string()),
        ..Default::default()
    }
}
```

#### 5.4 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tabs() {
        // 测试 Tab 列表
    }

    #[test]
    fn test_send_message() {
        // 测试消息发送
    }

    #[test]
    fn test_see_output() {
        // 测试屏幕读取
    }

    #[test]
    fn test_open_close_tab() {
        // 测试 Tab 生命周期
    }

    #[test]
    fn test_id_mapping() {
        // 测试 1-based ↔ 0-based ID 映射
    }
}
```

#### 5.5 多实例测试

```bash
# 终端 1: 启动 WezTeam
cargo run --release

# 终端 2: 启动多个 tsh 实例
tsh open "Agent1" claude --dangerously-skip-permissions
tsh open "Agent2" codex --dangerously-skip-permissions

# 验证 Agent 间通信
tsh send 2 "hello from Agent1"
tsh see 2
```

**产出**: 稳定可用的 WezTeam 版本。

---

## 5. 关键技术决策

### 5.1 绕过二进制 PDU 协议

**决策**: 不扩展 WezTerm 的 `codec` crate，直接在 `wezterm-mux-server-impl` 上层实现 JSON handler。

**理由**:
- WezTerm 的 PDU 协议有 60+ 消息类型，版本化，压缩
- TeamShell 只需要 7 个命令
- JSON 更容易调试和维护
- 与现有 tsh CLI 兼容

### 5.2 ID 映射策略

**决策**: WezTerm 内部用 0-based ID，TeamShell 外部用 1-based ID。

**实现**:
```rust
// 接收请求时: teamshell_id → wezterm_id
let wezterm_id = teamshell_id - 1;

// 返回响应时: wezterm_id → teamshell_id
let teamshell_id = wezterm_id + 1;
```

### 5.3 运行时选择

**决策**: 使用 tokio 作为 IPC 运行时。

**理由**:
- WezTerm 使用 smol，但 TeamShell 需要 tokio 的 Named Pipe 支持
- 可以在独立线程中运行 tokio runtime
- 不影响 WezTerm 主循环

**实现**:
```rust
// 在 wezterm-gui 的 main 函数中启动 tokio runtime
std::thread::spawn(|| {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(teamshell::server::start_server(mux));
});
```

### 5.4 tsh 输出格式

**决策**: tsh 输出纯文本，不输出 JSON。

**理由**:
- 大模型更易解析
- Token 更省
- 人类也更易读

**实现**: 在 `tsh/src/commands.rs` 中修改输出逻辑。

---

## 6. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| smol vs tokio 运行时冲突 | 中 | 在独立线程运行 tokio，不干扰 WezTerm 主循环 |
| WezTerm 上游更新合并困难 | 高 | 保持 fork 精简，只改必要部分；定期 rebase |
| Tab ID 映射错误 | 中 | 单元测试覆盖所有 ID 转换场景 |
| Windows Named Pipe 权限问题 | 低 | 使用默认安全属性，必要时添加 ACL |
| 性能瓶颈 | 低 | JSON 解析开销可忽略；PTY I/O 是主要瓶颈 |

---

## 7. 文件修改清单

### 新增文件

```
wezterm-mux-server-impl/src/teamshell/
├── mod.rs
├── protocol.rs
├── server.rs
├── handler.rs
└── output.rs
```

### 修改文件

```
wezterm-mux-server-impl/Cargo.toml    # 添加 tokio, serde_json 依赖
wezterm-mux-server-impl/src/lib.rs    # 添加 teamshell 模块
wezterm-gui/src/main.rs               # 启动 TeamShell 服务
mux/src/lib.rs                        # 添加 Tab 名字管理方法 (可选)
```

### 不修改文件

```
mux/src/pane.rs                       # Pane trait 已有所需方法
mux/src/localpane.rs                  # LocalPane 已实现 Pane trait
term/                                 # 终端模拟层不需要改动
pty/                                  # PTY 层不需要改动
```

---

## 8. 验收标准

### 功能验收

- [ ] `tsh list` 显示所有 Tab (ID + Name)
- [ ] `tsh open "name" cmd` 创建新 Tab
- [ ] `tsh close <id>` 关闭 Tab
- [ ] `tsh send <id> "msg"` 发送文本到 Tab
- [ ] `tsh send <id> --key "\x03"` 发送按键到 Tab
- [ ] `tsh see <id>` 读取 Tab 屏幕内容
- [ ] `tsh name <id> "new_name"` 改名 Tab
- [ ] Agent 间消息自动添加 `[TeamShell 消息]` 前缀
- [ ] `TEAMSH_TAB_ID` 和 `TEAMSH_NAME` 环境变量正确注入

### 性能验收

- [ ] Tab 创建 < 100ms
- [ ] 消息发送 < 50ms
- [ ] 屏幕读取 < 100ms
- [ ] 支持 10+ Tab 同时运行

### 兼容性验收

- [ ] Windows 10/11 正常运行
- [ ] Linux (Ubuntu 20.04+) 正常运行
- [ ] macOS 正常运行
- [ ] 与现有 tsh CLI 完全兼容

---

## 9. 时间估算

| 阶段 | 工作量 | 预计时间 |
|------|--------|----------|
| Phase 1: 基础设施 | 编译验证 + 清理 | 2-3 天 |
| Phase 2: 协议 Crate | IPC 协议实现 | 3-5 天 |
| Phase 3: Tab 管理 | 生命周期 + 环境变量 | 2-3 天 |
| Phase 4: tsh 适配 | CLI 输出格式 | 1-2 天 |
| Phase 5: 完善测试 | 测试 + 修复 | 2-3 天 |
| **总计** | | **10-16 天** |

---

## 10. 后续扩展

### 10.1 GUI 集成

- Tab 栏显示 TeamShell Tab 名字
- 状态栏显示 Agent 状态
- 消息通知（当有新消息时高亮 Tab）

### 10.2 高级功能

- Tab 分组（项目维度）
- 消息历史记录
- 文件传输
- 剪贴板共享

### 10.3 性能优化

- 屏幕快照增量更新
- 消息批量发送
- 连接池复用
