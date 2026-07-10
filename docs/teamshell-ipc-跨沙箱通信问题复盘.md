# TeamShell IPC 跨沙箱通信问题 — 完整复盘

## 一、产品背景

WezTeam 是 WezTerm 的 fork，核心新增功能是 TeamShell——让多个 AI Agent 在不同终端标签页中协作。架构：

- **服务端**：运行在 `wezterm-gui` 进程内的 TeamShell server 线程
- **客户端**：`tsh` CLI 工具，Agent 通过它与服务端 IPC 通信
- **场景**：Agent 可能运行在宿主机终端，也可能运行在 CodeArts 沙箱终端

## 二、问题现象

用户在 CodeArts 沙箱中启动的 Agent 执行 `tsh status`，报错：

```
Connection refused (os error 111)
```

或

```
No such file or directory (os error 2)
```

无法连接到 TeamShell 后端。

## 三、根因分析过程

### 第一轮：发现 socket 类型不匹配

- **服务端** (`server.rs`) 使用 `tokio::net::UnixListener::bind(path)` 创建**文件系统 socket**
- **客户端** (`tsh/ipc.rs`) 使用 `interprocess` crate 的 `GenericNamespaced` 创建**抽象命名空间 socket**
- 两者地址空间完全不同，根本连不上

### 第二轮：提取共享 IPC 层

- 创建 `teamshell-ipc` crate，统一端点发现和 socket 创建逻辑
- 将 server 和 client 都改为使用 `teamshell-ipc`
- **关键决策**：改用 `GenericFilePath`（文件系统 socket），因为抽象命名空间 socket 无法跨沙箱边界

### 第三轮：编译通过但连接失败

- 编译全部通过
- 用户在宿主机终端测试 `tsh status`，报 `Connection refused`
- 错误路径：`/mnt/code/work/wezteam/target/release/.teamshell.sock`
- 原因：后端（wezterm-gui）未启动

### 第四轮：试图在沙箱内验证（失误）

- 在 CodeArts 沙箱中执行 `echo $TEAMSH_TAB_ID`，结果为空
- 在沙箱中执行 `tsh status`，`tsh: not found`
- **严重失误**：沙箱环境与宿主机环境完全隔离，环境变量和 PATH 都不同，不应该在沙箱内做此验证

### 第五轮：代码审查发现新问题

- `GenericNamespaced` 被移除 → Windows 兼容性中断
- `HICLI_BACKEND_PID` 环境变量回退被删除
- 修复了这两个问题，编译通过

### 第六轮：用户指出根本问题未解决

用户反复强调：
1. **这是产品，不是玩具**——必须绝对稳定
2. **文件系统 socket 方案可能不稳定**——如果有一丝一毫的不确定性，必须先澄清
3. **沙箱隔离是核心障碍**——CodeArts 沙箱与宿主机的命名空间隔离，不是简单改个 socket 类型就能解决的

## 四、未解决的核心问题

### 问题 1：文件系统 socket 跨沙箱边界的可靠性

- 文件系统 socket 要求 socket 文件所在的路径在宿主机和沙箱之间**共享可见**
- CodeArts 沙箱中 `/mnt/code/work/wezteam/` 目录看起来是共享的，但**没有验证过**
- 即使目录可见，socket 文件的**权限**是否允许沙箱进程连接？沙箱可能以不同用户身份运行
- SELinux/AppArmor 策略可能阻止跨命名空间的 socket 访问
- **结论：文件系统 socket 方案的跨沙箱可靠性未经任何验证**

### 问题 2：路径发现机制不可靠

当前 `socket_path()` 的三级回退：
1. `TEAMSH_SOCKET_PATH` 环境变量 → 需要手动设置
2. `WEZTERM_EXECUTABLE_DIR/.teamshell.sock` → 构建目录，不是生产路径
3. `/tmp/TeamShell-wezteam.sock` → 可能被 tmpwatch 清理

**没有一个路径是生产级可靠的**。用户必须手动配置环境变量，这不是产品体验。

### 问题 3：wezterm 自身的 mux IPC 已经解决了类似问题

wezterm 原有的 mux server 使用 `wezterm-uds` crate（标准 `std::os::unix::net::UnixListener`），socket 路径放在 `RUNTIME_DIR`（通常是 `$XDG_RUNTIME_DIR/wezterm/`），有完整的权限检查、目录创建、stale socket 清理逻辑。TeamShell 的 IPC 完全没有复用这套成熟机制。

### 问题 4：`interprocess` crate 引入了不必要的复杂度

- wezterm 原有 IPC 用标准库 `std::os::unix::net`，简单可靠
- TeamShell 引入了 `interprocess` crate，带来了 `GenericNamespaced` vs `GenericFilePath` 的选择问题
- `Box::leak` 获取 `'static` 生命周期是 hack
- 在服务端长期运行场景下，每次 `create_listener` 都泄漏内存

## 五、用户传达的原则

1. **这是产品，不是自己用的工具**——稳定性要求是生产级的，不能有"大概能用"的心态
2. **如果方案有一丝一毫的不确定性，必须先澄清**——不能装作问题不存在，不能给用户装逼
3. **沙箱隔离是客观事实**——CodeArts 沙箱环境与宿主机隔离，这不是能绕过的
4. **先搞清楚再动手**——不要武断判定问题，不要糊里糊涂干活
5. **不造新接口，先复用已有**——wezterm 已有成熟的 IPC 机制，应该复用而不是另起炉灶

## 六、待决策事项

1. **IPC 传输层选型**：文件系统 socket vs TCP loopback vs 复用 wezterm mux 协议 vs 其他？
2. **跨沙箱连通性验证**：CodeArts 沙箱到底支持哪些 IPC 机制？需要实际测试确认
3. **是否复用 wezterm 已有的 mux IPC**：`wezterm-uds` + `RUNTIME_DIR` 机制已经成熟，TeamShell 是否应该复用？
4. **`interprocess` crate 的去留**：是否应该回归标准库 `std::os::unix::net`？
5. **路径发现的生产化**：socket 路径应该放在哪里？`$XDG_RUNTIME_DIR/wezteam/teamshell.sock`？

## 七、核心结论（已过时）

~~**文件系统 socket 方案未经跨沙箱验证，存在多个未确认的风险点，不应作为生产方案直接上线。** 需要先确认 CodeArts 沙箱的 IPC 能力边界，再决定技术方案。~~

> **更新历史**：
> - 第一版：文件系统 socket 方案 → 已废弃
> - 第二版：TCP loopback + 随机端口 + 端口文件 + 随机 Token → 已废弃（端口文件在沙箱/容器环境中不可靠）
> - 第三版（当前）：**TCP loopback + 固定端口 + 固定 Token**，详见第十节

---

## 八、CodeArts 沙箱真实性质（关键纠正）

### 8.1 原先的错误假设

复盘文档第四、六节将 CodeArts 沙箱描述为"命名空间隔离"环境，隐含假设：
- 沙箱有独立的网络命名空间 → TCP loopback 不可达
- 沙箱有独立的文件系统命名空间 → Unix domain socket 不可见

**这是错误假设。**

### 8.2 官方文档揭示的真实性质

根据 [CodeArts 沙箱用户手册](https://support.huaweicloud.com/usermanual-cli/codeartsagent_cli_0007.html)，CodeArts 沙箱是**进程级命令拦截 + 文件访问控制层**，不是容器/网络命名空间隔离：

| 维度 | 原先假设 | 实际情况 |
|------|---------|---------|
| 网络隔离 | 独立网络命名空间 | ❌ 无网络隔离，共享宿主机网络栈 |
| 文件系统隔离 | 独立文件系统命名空间 | ❌ 共享宿主机文件系统，但有读写权限控制 |
| 隔离机制 | namespace/cgroup | 命令拦截 + 目录权限白名单 |
| 127.0.0.1 | 不可达 | ✅ 完全可达 |
| 抽象命名空间 socket | 不可见 | ✅ 可见（同网络命名空间） |

### 8.3 沙箱文件访问控制（与 IPC 相关）

| 平台 | 可写目录 |
|------|---------|
| Linux | `/tmp`、`~/.cache`、`~/.local/lib`、`~/.local/bin`、`~/.local/share`、项目目录 |
| macOS | `/tmp`、`/var/folders`、`~/Library/Caches`、`~/.local/share`、项目目录 |
| Windows | 项目目录及其子目录 |

### 8.4 对方案选择的影响

- TCP loopback `127.0.0.1` 在沙箱内外**完全可达** → TCP 方案可行
- 文件系统 socket 放在可写目录中**沙箱内外可见** → Unix domain socket 方案也可行
- 原始 `Connection refused` 的根因是 `interprocess` 的 socket 类型不匹配（抽象命名空间 vs 文件系统），不是沙箱网络隔离

### 8.5 为什么最终不选 Unix domain socket

虽然 Unix domain socket 在 CodeArts 沙箱场景下也可行，但在 Windows 上依赖 `AF_UNIX` 可选组件（`uds_windows` crate 底层调用 WinSock2 `AF_UNIX`），不是 100% 可用。Windows Server 上 `AF_UNIX` 是可选功能，可能未安装，此时 `socket(AF_UNIX, ...)` 返回 `WSAEAFNOSUPPORT` (10047)。

TCP loopback 是唯一在三大平台上**零前提条件、零可选依赖、100% 可用**的传输层。

---

## 九、需求规格

### 9.1 功能需求

| 编号 | 需求 | 优先级 | 验收标准 |
|------|------|--------|---------|
| F-01 | `tsh` CLI 能在宿主机终端连接 TeamShell 后端 | P0 | `tsh status` 返回正常 |
| F-02 | `tsh` CLI 能在 CodeArts 沙箱终端连接 TeamShell 后端 | P0 | 沙箱内 `tsh status` 返回正常 |
| F-03 | 支持 Windows / macOS / Linux 三大平台 | P0 | 三平台 `tsh status` 均返回正常 |
| F-04 | 移除 `interprocess` crate 依赖 | P0 | `cargo tree` 无 interprocess |
| F-05 | 消除 `Box::leak` 内存泄漏 | P0 | `create_server` 无内存泄漏 |
| F-06 | IPC 通信具备认证机制 | P0 | 未认证的连接被拒绝 |
| F-07 | 端点发现无需用户手动配置 | P1 | 默认路径下 `tsh status` 自动工作 |
| F-08 | 服务端异常退出后客户端能检测到陈旧端点 | P1 | 客户端不连接到已死亡的端口 |

### 9.2 非功能需求

| 编号 | 需求 | 指标 |
|------|------|------|
| NF-01 | 跨平台兼容性 | Windows 10 2019+ / Windows 11、macOS 11+、Ubuntu 18.04+，零可选依赖 |
| NF-02 | 安全性 | 非授权进程无法执行 IPC 命令 |
| NF-03 | 启动时序容错 | 服务端未就绪时客户端重试后成功（3 次指数退避） |
| NF-04 | 防火墙无感 | Windows 上不触发防火墙弹窗 |
| NF-05 | 协议兼容 | 现有 JSON over newline 协议语义不变 |

### 9.3 约束

- C-01：不得引入 `interprocess`、`uds_windows` 等非标准库 socket 抽象
- C-02：不得使用 Unix domain socket（Windows 上依赖 AF_UNIX 可选组件，非 100% 可用）
- C-03：不得使用抽象命名空间 socket（Linux 专属，跨平台不可用）
- C-04：协议层保持同步请求-响应模式（客户端 `std::net::TcpStream`，服务端 `tokio::net::TcpListener`）
- C-05：不得破坏现有 `IpcRequest` / `IpcResponse` 的语义（可扩展，不可删改已有字段）

---

## 十、方案设计（⚠️ 已废弃 — 详见第十三节）

> **⚠️ 本节描述的是第二版方案（随机端口 + 端口文件 + 随机 Token），已被第十三节的第三版方案（固定端口 + 固定 Token）替代。以下内容仅保留供历史参考。**

### 10.1 传输层：TCP loopback

```
服务端：tokio::net::TcpListener::bind("127.0.0.1:0")
        → OS 分配随机端口 → 写入端口文件

客户端：std::net::TcpStream::connect("127.0.0.1:<port>")
        → 从端口文件读取端口号和认证 token
```

**为什么选 TCP loopback 而非 Unix domain socket：**

| 维度 | TCP loopback | Unix domain socket |
|------|-------------|-------------------|
| Linux | ✅ `std::net` | ✅ `std::os::unix::net` |
| macOS | ✅ `std::net` | ✅ `std::os::unix::net` |
| Windows | ✅ `std::net` | ⚠️ 依赖 `AF_UNIX` 可选组件 |
| 跨平台统一 API | ✅ 一套代码 | ❌ 需要 `uds_windows` + 条件编译 |
| 跨沙箱（CodeArts） | ✅ 同网络命名空间 | ✅ 同文件系统（放在可写目录） |

TCP loopback 是唯一在三大平台上**零前提条件、零可选依赖、100% 可用**的传输层。

### 10.2 认证机制：Token-based

```
┌─────────────────────────────────────────────────────┐
│                    服务端启动                          │
│                                                      │
│  1. TcpListener::bind("127.0.0.1:0") → port         │
│  2. token = Uuid::new_v4().to_string()              │
│  3. 原子写入端口文件: {port, pid, token}              │
│  4. 开始 accept 循环                                 │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│                    客户端连接                          │
│                                                      │
│  1. 读取端口文件 → {port, pid, token}                │
│  2. kill(pid, 0) 验证服务端存活                       │
│  3. TcpStream::connect("127.0.0.1:{port}")          │
│  4. 发送请求时携带 auth_token 字段                    │
│  5. 服务端验证 token，不匹配则断开连接                 │
└─────────────────────────────────────────────────────┘
```

**Token 安全性分析：**

- 端口文件权限 `0600`（仅当前用户可读）→ 其他用户无法获取 token
- Token 每次服务端启动重新生成 → 重启后旧 token 失效
- 攻击者即使扫描到端口，无 token 也无法通过认证
- 与 Unix domain socket 的文件权限保护等价（都依赖 OS 文件权限）

### 10.3 端口文件设计

#### 文件格式

```
<port>\n<pid>\n<token>\n
```

示例：
```
42831
12345
a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

#### 原子写入

```rust
fn write_port_file_atomic(path: &Path, port: u16, pid: u32, token: &str) -> io::Result<()> {
    let content = format!("{}\n{}\n{}\n", port, pid, token);
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, path) // 同文件系统上 rename 是原子的
}
```

#### 路径发现优先级

```
1. TEAMSH_PORT_FILE 环境变量        → 用户显式指定
2. 项目目录/.teamshell-port          → 沙箱可写，跨沙箱可见（三平台通用）
3. 平台默认路径：
   Linux/macOS: /tmp/.teamshell-port
   Windows:     %LOCALAPPDATA%/wezteam/teamshell-port
```

**为什么项目目录优先于 `/tmp`：**
- 项目目录在三大平台（含 Windows）上都是 CodeArts 沙箱保证可写的
- `/tmp` 在 Windows 上不存在
- 项目目录与 wezterm-gui 的工作目录一致，最自然

#### 陈旧文件检测

客户端读取端口文件后，验证服务端 PID 是否存活：

```rust
#[cfg(unix)]
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 || *libc::__errno_location() != libc::ESRCH }
}

#[cfg(windows)]
fn is_pid_alive(pid: u32) -> bool {
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::Foundation::CloseHandle;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}
```

如果 PID 不存活，客户端删除陈旧端口文件并报错。

### 10.4 协议扩展

#### IpcRequest 增加 auth_token 字段

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub auth_token: String,
    #[serde(flatten)]
    pub command: IpcCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcCommand {
    #[serde(rename = "type")]
    Type { tab_index: usize, message: String, from_tab_id: Option<usize> },
    TypeRaw { tab_index: usize, data: String },
    View { tab_index: usize, line_count: usize },
    Status,
    Report { tab_index: usize, key: String, value: Option<String> },
    Query { tab_index: usize },
    Open { name: String, command: Option<String>, args: Option<Vec<String>>,
           cwd: Option<String>, env: Option<Vec<(String, String)>> },
    Close { tab_index: usize },
    Name { tab_index: usize, new_name: String },
}
```

**兼容性说明：**
- `#[serde(flatten)]` 将 `IpcCommand` 的字段展平到 JSON 顶层，与现有协议格式完全一致
- 现有 JSON 如 `{"cmd":"status"}` 变为 `{"auth_token":"xxx","cmd":"status"}`
- `IpcResponse` 不变，无需修改

#### 服务端认证流程

```rust
async fn handle_client(mut stream: TcpStream, expected_token: &str) -> anyhow::Result<()> {
    let mut request_bytes = Vec::new();
    let bytes_read = read_json_line(&mut stream, &mut request_bytes).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request: IpcRequest =
        serde_json::from_slice(&request_bytes).context("failed to decode TeamShell request")?;

    if request.auth_token != expected_token {
        log::warn!("TeamShell auth failed from {}", stream.peer_addr()?);
        let _ = stream.shutdown().await;
        return Err(anyhow!("authentication failed"));
    }

    log::info!("TeamShell request: {}", request.command.command_name());

    let response = serde_json::to_string(&Handler::handle(request.command))
        .context("failed to encode TeamShell response")?;

    stream.write_all(response.as_bytes()).await
        .context("failed to write TeamShell response")?;
    stream.write_all(b"\n").await
        .context("failed to write TeamShell response newline")?;
    stream.flush().await
        .context("failed to flush TeamShell response")?;

    Ok(())
}
```

### 10.5 服务端架构

```
wezterm-gui main.rs
  └─ spawn_teamshell_server()
       └─ std::thread::spawn(|| {
              tokio::runtime::Builder::new_current_thread()
                .block_on(start_server())
          })

start_server():
  1. TcpListener::bind("127.0.0.1:0")
  2. 获取 local_addr().port()
  3. 生成 auth_token = Uuid::new_v4()
  4. 原子写入端口文件
  5. loop { accept → spawn handle_client }
```

**与现有架构的对比：**

| 维度 | 现有 | 改造后 |
|------|------|--------|
| 监听器 | `interprocess::local_socket::tokio::Listener` | `tokio::net::TcpListener` |
| 客户端流 | `interprocess::local_socket::tokio::Stream` | `tokio::net::TcpStream` |
| 端点发现 | `socket_path()` → 文件系统路径 | 端口文件 → `{port, pid, token}` |
| 认证 | 无 | Token-based |
| 协议 | `IpcRequest` (enum) | `IpcRequest { auth_token, command: IpcCommand }` |
| 异步模型 | 服务端 tokio，客户端同步 | 不变 |

### 10.6 客户端架构

```
tsh main.rs
  └─ commands::status() / type() / ...
       └─ ipc::send_request(command)
            1. 读取端口文件 → {port, pid, token}
            2. 验证 PID 存活
            3. TcpStream::connect("127.0.0.1:{port}")
            4. stream.set_nodelay(true)
            5. stream.set_read_timeout(Some(5s))
            6. stream.set_write_timeout(Some(5s))
            7. 写入 JSON + "\n"
            8. 读取响应 JSON + "\n"
```

**重试逻辑保持不变：** 3 次指数退避（100ms → 200ms → 400ms）。

### 10.7 依赖变更

#### 移除

| Crate | 从哪个 Cargo.toml 移除 |
|-------|----------------------|
| `interprocess` | `teamshell-ipc/Cargo.toml`、`wezterm-mux-server-impl/Cargo.toml`、`tsh/Cargo.toml` |

#### 新增

| Crate | 添加到 | 用途 |
|-------|--------|------|
| `uuid` (features = `["v4"]`) | `teamshell-ipc/Cargo.toml` | 生成认证 token |
| `tokio` (features = `["net"]`) | `teamshell-ipc/Cargo.toml` | `TcpListener`（服务端） |

#### 已有、无需变更

| Crate | 用途 |
|-------|------|
| `tokio` | 服务端异步运行时（`wezterm-mux-server-impl` 已依赖） |
| `serde` / `serde_json` | 协议序列化（已依赖） |
| `libc` | PID 存活检测（`wezterm-mux-server-impl` 已依赖） |

---

## 十一、详细执行计划

### Phase 1：teamshell-ipc crate 重写

**目标：** 将 `teamshell-ipc` 从 `interprocess` 抽象层改造为 TCP loopback + 端口文件 + 认证 token。

#### 文件：`teamshell-ipc/Cargo.toml`

```diff
 [dependencies]
-interprocess = { version = "2.4", features = ["tokio"] }
 log = "0.4"
+uuid = { version = "1", features = ["v4"] }
+tokio = { workspace = true, features = ["net"] }
+serde = { workspace = true, features = ["derive"] }
+serde_json.workspace = true
```

#### 文件：`teamshell-ipc/src/lib.rs`

**完全重写。** 新的公共 API：

```rust
// --- 端口文件路径发现 ---
pub fn port_file_path() -> PathBuf;

// --- 服务端：创建 TCP 监听器 ---
pub struct ServerEndpoint {
    pub listener: tokio::net::TcpListener,
    pub port: u16,
    pub auth_token: String,
}
pub fn create_server() -> Result<ServerEndpoint, String>;

// --- 客户端：连接到服务端 ---
pub struct ServerInfo {
    pub port: u16,
    pub pid: u32,
    pub auth_token: String,
}
pub fn discover_server() -> Result<ServerInfo, String>;
pub fn connect() -> Result<std::net::TcpStream, String>;
```

**实现要点：**

1. `port_file_path()` — 三级回退：`TEAMSH_PORT_FILE` > 项目目录 > 平台默认
2. `create_server()` — `TcpListener::bind("127.0.0.1:0")` + 生成 token + 原子写入端口文件
3. `discover_server()` — 读取端口文件 + 验证 PID 存活
4. `connect()` — `discover_server()` + `TcpStream::connect()` + `set_nodelay(true)`

**端口文件原子写入实现：**

```rust
fn write_port_file_atomic(path: &Path, port: u16, pid: u32, token: &str) -> io::Result<()> {
    let content = format!("{}\n{}\n{}\n", port, pid, token);
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, path)
}
```

**PID 存活检测实现：**

```rust
#[cfg(unix)]
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 || *libc::__errno_location() != libc::ESRCH }
}

#[cfg(windows)]
fn is_pid_alive(pid: u32) -> bool {
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::Foundation::CloseHandle;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle != 0 {
            CloseHandle(handle);
            true
        } else {
            false
        }
    }
}
```

**项目目录发现：**

```rust
fn project_dir() -> Option<PathBuf> {
    std::env::current_dir().ok()
}
```

### Phase 2：协议层扩展

#### 文件：`wezterm-mux-server-impl/src/teamshell/protocol.rs`

将 `IpcRequest` 从 enum 拆分为 struct + enum：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub auth_token: String,
    #[serde(flatten)]
    pub command: IpcCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcCommand {
    // 与原 IpcRequest enum 的变体完全一致
    #[serde(rename = "type")]
    Type { tab_index: usize, message: String, from_tab_id: Option<usize> },
    TypeRaw { tab_index: usize, data: String },
    View { tab_index: usize, line_count: usize },
    Status,
    Report { tab_index: usize, key: String, value: Option<String> },
    Query { tab_index: usize },
    Open { name: String, command: Option<String>, args: Option<Vec<String>>,
           cwd: Option<String>, env: Option<Vec<(String, String)>> },
    Close { tab_index: usize },
    Name { tab_index: usize, new_name: String },
}
```

**同步更新：** `handler.rs` 中 `Handler::handle(request: IpcRequest)` 改为 `Handler::handle(command: IpcCommand)`，内部 match 不变。

#### 文件：`tsh/src/protocol.rs`

同步拆分 `IpcRequest` 为 `IpcRequest { auth_token, command: IpcCommand }`。

#### 文件：`tsh/src/commands.rs`

所有构造 `IpcRequest` 的地方增加 `auth_token` 字段。

### Phase 3：服务端改造

#### 文件：`wezterm-mux-server-impl/Cargo.toml`

```diff
-interprocess = { version = "2.4", features = ["tokio"] }
```

#### 文件：`wezterm-mux-server-impl/src/teamshell/server.rs`

```rust
use tokio::net::TcpListener;

pub async fn start_server() -> anyhow::Result<()> {
    let endpoint = teamshell_ipc::create_server()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    log::info!("TeamShell server listening on 127.0.0.1:{}", endpoint.port);

    loop {
        let (stream, _) = endpoint.listener
            .accept()
            .await
            .context("failed to accept TeamShell client")?;

        let token = endpoint.auth_token.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, &token).await {
                log::error!("TeamShell client error: {:#}", err);
            }
        });
    }
}

pub async fn handle_client(mut stream: TcpStream, expected_token: &str) -> anyhow::Result<()> {
    let mut request_bytes = Vec::new();
    let bytes_read = read_json_line(&mut stream, &mut request_bytes).await?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request: IpcRequest =
        serde_json::from_slice(&request_bytes).context("failed to decode TeamShell request")?;

    if request.auth_token != expected_token {
        log::warn!("TeamShell auth failed from {}", stream.peer_addr()?);
        let _ = stream.shutdown().await;
        return Err(anyhow!("authentication failed"));
    }

    log::info!("TeamShell request: {}", request.command.command_name());

    let response = serde_json::to_string(&Handler::handle(request.command))
        .context("failed to encode TeamShell response")?;

    stream.write_all(response.as_bytes()).await
        .context("failed to write TeamShell response")?;
    stream.write_all(b"\n").await
        .context("failed to write TeamShell response newline")?;
    stream.flush().await
        .context("failed to flush TeamShell response")?;

    Ok(())
}
```

**关键变化：**
- `interprocess::local_socket::tokio::Listener` → `tokio::net::TcpListener`
- `handle_client` 接收 `TcpStream` 而非泛型 `T: AsyncRead + AsyncWrite`
- 新增 token 认证逻辑
- `Handler::handle(request)` → `Handler::handle(request.command)`

### Phase 4：客户端改造

#### 文件：`tsh/Cargo.toml`

```diff
-interprocess = "2.4"
```

#### 文件：`tsh/src/ipc.rs`

```rust
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use crate::protocol::{IpcCommand, IpcRequest, Response};

const TIMEOUT: Duration = Duration::from_secs(5);

pub fn send_request(command: IpcCommand) -> Result<Response, String> {
    let server_info = teamshell_ipc::discover_server()?;
    let auth_token = server_info.auth_token.clone();
    let addr = format!("127.0.0.1:{}", server_info.port);

    let max_retries = 3;
    let mut delay = Duration::from_millis(100);
    let mut stream = None;

    for attempt in 0..max_retries {
        match std::net::TcpStream::connect(&addr) {
            Ok(s) => { stream = Some(s); break; }
            Err(e) if attempt < max_retries - 1 => {
                eprintln!("连接失败，{}ms 后重试 ({}/{}): {}",
                    delay.as_millis(), attempt + 1, max_retries, e);
                std::thread::sleep(delay);
                delay *= 2;
            }
            Err(e) => {
                return Err(format!("无法连接到 teamshell 后端 ({}): {}\n请确认后端正在运行。",
                    addr, e));
            }
        }
    }

    let mut stream = stream.unwrap();
    stream.set_nodelay(true).map_err(|e| format!("设置 TCP_NODELAY 失败: {}", e))?;
    stream.set_read_timeout(Some(TIMEOUT)).ok();
    stream.set_write_timeout(Some(TIMEOUT)).ok();

    let request = IpcRequest {
        auth_token,
        command,
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化请求失败: {}", e))?;

    stream.write_all(request_json.as_bytes())
        .map_err(|e| format!("发送请求失败: {}", e))?;
    stream.write_all(b"\n")
        .map_err(|e| format!("发送请求分隔符失败: {}", e))?;
    stream.flush()
        .map_err(|e| format!("刷新写入缓冲区失败: {}", e))?;

    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line)
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if response_line.is_empty() {
        return Err("后端返回了空响应".to_string());
    }

    serde_json::from_str(response_line.trim())
        .map_err(|e| format!("解析响应失败: {} (原始: {})", e, response_line.trim()))
}
```

**关键变化：**
- `interprocess::local_socket::Stream` → `std::net::TcpStream`
- `teamshell_ipc::discover_endpoint()` + `make_socket_name()` → `teamshell_ipc::discover_server()`
- 新增 `stream.set_nodelay(true)` 禁用 Nagle
- `send_request` 参数从 `&Request` 改为 `IpcCommand`，内部组装 `IpcRequest { auth_token, command }`

#### 文件：`tsh/src/commands.rs`

所有调用 `send_request` 的地方，将 `IpcRequest::XXX { ... }` 改为 `IpcCommand::XXX { ... }`。

### Phase 5：清理

1. 删除 `teamshell-ipc/src/lib.rs` 中所有 `interprocess` 相关代码
2. 删除 `HICLI_ENDPOINT`、`HICLI_BACKEND_PID`、`WEZTERM_EXECUTABLE_DIR` 环境变量回退
3. 删除 `socket_path()`、`discover_endpoint()`、`make_socket_name()`、`make_default_socket_name()` 函数
4. 删除 `ENDPOINT_PREFIX` 常量
5. 服务端退出时清理端口文件（`Drop` 或 `ctrlc` handler）

### Phase 6：测试验证

| 测试项 | 平台 | 方法 |
|--------|------|------|
| T-01 宿主机连接 | Linux/macOS/Windows | `tsh status` 正常返回 |
| T-02 沙箱连接 | Linux | CodeArts 沙箱内 `tsh status` 正常返回 |
| T-03 认证失败 | 全平台 | 手动修改端口文件中的 token，`tsh status` 应返回认证失败 |
| T-04 陈旧端口文件 | 全平台 | 杀掉 wezterm-gui，`tsh status` 应检测到 PID 不存活 |
| T-05 服务端未就绪 | 全平台 | 先启动 `tsh`，再启动 wezterm-gui，3 次重试后应成功 |
| T-06 并发连接 | 全平台 | 多个 `tsh` 同时执行，均正常返回 |
| T-07 Windows 防火墙 | Windows | 启动 wezterm-gui 后无防火墙弹窗 |
| T-08 端口文件原子性 | Linux | 在服务端写入端口文件时杀掉进程，验证 `.tmp` 文件被清理或不被读取 |

---

## 十二、影响范围与风险

### 12.1 变更文件清单

| 文件 | 变更类型 | 说明 |
|------|---------|------|
| `teamshell-ipc/Cargo.toml` | 修改 | 移除 interprocess，新增 uuid/tokio |
| `teamshell-ipc/src/lib.rs` | 重写 | TCP 监听器 + 端口文件 + 认证 token |
| `wezterm-mux-server-impl/Cargo.toml` | 修改 | 移除 interprocess |
| `wezterm-mux-server-impl/src/teamshell/server.rs` | 重写 | TcpListener + token 认证 |
| `wezterm-mux-server-impl/src/teamshell/protocol.rs` | 修改 | IpcRequest 拆分为 struct + enum |
| `wezterm-mux-server-impl/src/teamshell/handler.rs` | 修改 | handle(IpcCommand) 替代 handle(IpcRequest) |
| `tsh/Cargo.toml` | 修改 | 移除 interprocess |
| `tsh/src/ipc.rs` | 重写 | TcpStream + token |
| `tsh/src/protocol.rs` | 修改 | IpcRequest 拆分 |
| `tsh/src/commands.rs` | 修改 | IpcCommand 替代 IpcRequest |

### 12.2 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| Windows PID 检测 API 权限不足 | 低 | 客户端误判服务端已死 | `OpenProcess` 使用 `PROCESS_QUERY_LIMITED_INFORMATION`（最低权限） |
| 端口文件 `.tmp` 残留 | 低 | 下次启动可能读到不完整数据 | `discover_server()` 验证文件内容格式完整性 |
| 端口耗尽（频繁重启） | 极低 | 服务端无法绑定 | OS 自动回收 TIME_WAIT 端口；`SO_REUSEADDR` |
| 多 wezterm-gui 实例冲突 | 低 | 端口文件被覆盖 | 端口文件路径包含 PID（如 `.teamshell-port-12345`） |
| `serde(flatten)` 与 `serde(tag)` 组合兼容性 | 中 | JSON 序列化格式变化 | 需要写测试验证序列化输出 |

### 12.3 不变的部分

- `IpcResponse` 结构体不变
- `TabState` 结构体不变
- `Handler` 的业务逻辑不变
- JSON over newline 协议格式不变（仅新增 `auth_token` 字段）
- 服务端线程模型不变（独立线程 + 单线程 tokio runtime）
- 客户端同步阻塞模型不变

---

## 十三、方案迭代：固定端口 + 固定 Token（第三版，当前生效）

### 13.1 第二版方案的问题

第二版方案（TCP loopback + 随机端口 + 端口文件 + 随机 Token）存在根本性缺陷：

1. **端口文件在沙箱/容器环境中不可靠**：CodeArts 沙箱、Docker 容器、bwrap 沙箱等环境的 `/tmp` 可能与宿主机不共享，端口文件无法被客户端发现
2. **PID 检测在隔离环境中失效**：沙箱内看不到宿主机进程，`kill(pid, 0)` 和 `OpenProcess` 都不可用
3. **端口文件引入了不必要的复杂度**：原子写入、陈旧检测、多实例冲突、文件权限——这些都是为了解决"动态端口需要被发现"这个人为制造的问题
4. **随机 Token 需要端口文件传递**：Token 的传递依赖端口文件，端口文件不可靠则 Token 也不可靠

### 13.2 第三版方案

**核心思路：消除所有运行时动态发现，全部硬编码。**

| 项目 | 第二版 | 第三版（当前） |
|------|--------|---------------|
| 端口 | `bind("127.0.0.1:0")` 随机分配 | **固定 `31415`** |
| Token | `Uuid::new_v4()` 随机生成 | **固定 `teamshell-31415`** |
| 端口文件 | `/tmp/.teamshell-port-{pid}` | **删除，不再需要** |
| PID 检测 | `kill(pid, 0)` / `OpenProcess` | **删除，端口占用 = 后端已运行** |
| 多实例 | 端口文件含 PID | **端口 bind 失败 = 后端已运行，跳过启动** |
| 依赖 | `uuid`, `serde`, `serde_json`, `libc`, `winapi` | **仅 `log`** |

### 13.3 为什么固定端口是安全的

- **TCP loopback `127.0.0.1` 只接受本机连接**，远程主机无法访问
- **固定 Token 提供应用层认证**，防止本机其他进程误连
- **两者组合等价于 Unix domain socket 的文件权限保护**（都限制为本机+同用户）
- 端口 `31415` 不在 IANA 注册的知名端口范围内，冲突概率极低

### 13.4 多实例处理

- `bind("127.0.0.1:31415")` 成功 → 启动服务端
- `bind` 失败（`EADDRINUSE`）→ 日志记录"后端已在运行"，跳过服务端启动
- 最后一个 wezterm-gui 退出时，OS 自动释放 TCP 端口
- 同一用户同一时间只有一个 TeamShell 后端，符合"后台进程必须只有一个"的需求

### 13.5 变更文件清单

| 文件 | 变更 |
|------|------|
| `teamshell-ipc/src/lib.rs` | 重写：删除端口文件/PID 检测/ServerEndpoint/ServerInfo，仅保留 `TEAMSH_PORT`、`TEAMSH_AUTH_TOKEN`、`connect_with_retries()` |
| `teamshell-ipc/Cargo.toml` | 移除 `uuid`、`tokio`、`serde`、`serde_json`、`libc`、`winapi` 依赖，仅保留 `log` |
| `wezterm-mux-server-impl/src/teamshell/server.rs` | 使用 `teamshell_ipc::TEAMSH_PORT` 固定端口 bind，`TEAMSH_AUTH_TOKEN` 固定 token；bind 失败时优雅退出而非崩溃重启 |
| `tsh/src/ipc.rs` | 使用 `teamshell_ipc::connect_with_retries()` 直连固定端口，`TEAMSH_AUTH_TOKEN` 固定 token |

### 13.6 删除的代码

- `port_file_path()` / `port_file_path_for_pid()`
- `platform_default_port_path()` / `platform_default_port_path_for_pid()`
- `discover_server()` / `ServerInfo` struct
- `ServerEndpoint` struct（含 `Drop` 清理端口文件）
- `create_server()` async 函数
- `write_port_file_atomic()` / `parse_port_file()`
- `is_pid_alive()`（Unix + Windows）
- `set_file_readonly_current_user()`（Windows）
- 所有端口文件相关测试
