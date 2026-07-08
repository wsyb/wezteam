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

## 七、核心结论

**文件系统 socket 方案未经跨沙箱验证，存在多个未确认的风险点，不应作为生产方案直接上线。** 需要先确认 CodeArts 沙箱的 IPC 能力边界，再决定技术方案。
