# WezTerm - AI Agent 团队协作终端

[English Documentation →](README.md)

<p align="center">
  <strong>WezTerm + TeamShell</strong>
</p>

<p align="center">
  在终端里组建 AI 团队，让多个 Agent 协同完成复杂任务
</p>

<p align="center">
  <a href="#快速开始">快速开始</a> ·
  <a href="#teamshell--多-agent-协作">TeamShell</a> ·
  <a href="#tsh-命令手册">tsh 命令</a> ·
  <a href="#消息处理">消息处理</a> ·
  <a href="#任务监管">任务监管</a> ·
  <a href="#垂直标签栏--信息面板">功能特性</a> ·
  <a href="#安装构建">安装构建</a> ·
  <a href="#配置说明">配置</a>
</p>

---

## 快速开始

### 启动第一个 Agent（推荐方式）

```bash
tsh open "我的助手" -- claude
```

后端会自动注入 `TEAMSH_TAB_ID`、`TEAMSH_NAME`、`TEAMSH_PLATFORM` 环境变量。

### 初始化项目协议

```bash
tsh init                      # 交互式：将协议写入检测到的 Agent 配置文件
tsh init --dry-run            # 预览变更，不写入
tsh init -y                   # 强制覆盖，不询问
tsh init --show               # 输出协议内容到 stdout
```

### 验证团队状态

```bash
tsh status    # 查看团队状态看板
tsh list      # 列出所有在职工位
```

---

## TeamShell — 多 Agent 协作

WezTeam 的核心创新是 **TeamShell**：一套让多个 AI Agent 在同一终端中协作工作的协议和工具链。

### 为什么需要 TeamShell？

单个 AI Agent 能力有限。当任务复杂时，你需要一个**团队**：前端专家、后端专家、测试工程师……TeamShell 让你在一个终端窗口中同时运行多个 Agent，它们可以：

- **互相观察** — 查看同事的屏幕输出
- **互相通信** — 给同事发消息、派任务
- **互相协调** — 自动分工、监管进度、汇报结果
- **动态扩编** — 随时招募新 Agent，任务完成后关闭

### 工作原理

每个 Agent 运行在一个**工位**（终端 Tab）中。`tsh`（TeamShell CLI）是管理工位的命令行工具：

```
┌─────────────────────────────────────────────┐
│  终端窗口 (WezTeam)                          │
│                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ 工位 1   │ │ 工位 2   │ │ 工位 3   │     │
│  │ 小明     │ │ 小红     │ │ 日志服务 │     │
│  │ (Agent)  │ │ (Agent)  │ │ (程序)   │     │
│  │          │ │          │ │          │     │
│  │ 正在写   │ │ 正在测   │ │ tail -f  │     │
│  │ 前端代码 │ │ 试接口   │ │ app.log  │     │
│  └──────────┘ └──────────┘ └──────────┘     │
│                                              │
│  tsh list  →  1 小明  2 小红  3 日志服务     │
│  tsh view 2  →  查看小红的屏幕               │
│  tsh type 2 "需要帮忙吗？"  →  给小红发消息  │
└─────────────────────────────────────────────┘
```

### 两种工位

| 类型 | 说明 | 交互方式 |
|------|------|---------|
| **Agent 工位**（活工位） | 运行带 TeamShell 协议的 LLM | 能识别消息、主动协作、回复同事 |
| **程序工位**（死工位） | 运行普通程序（Java/Vim/Shell） | 只能通过标准输入控制，不会主动响应 |

### 协议核心原则

1. **身份验证** — Agent 启动时通过 `TEAMSH_TAB_ID` + `tsh list` 双重验证，确保是正式团队成员
2. **消息识别** — `[TeamShell 消息] 来自 <名字>:` 前缀区分同事消息和程序输出
3. **自主决策** — Agent 有判断力，不需要每步请示，但不可逆操作需先问老板
4. **任务监管** — 谁派的活谁盯到底，问题逐层上报

协议全文见 [TeamShellProtocol.md](TeamShellProtocol.md) 和 [AGENTS.md](AGENTS.md)。

---

## 消息处理

### 怎么识别消息来源

当你接收到输入时，按以下规则判断来源：

| 来源 | 判定方法 | 含义 | 响应方式 |
|------|---------|------|---------|
| 用户直接输入 | 出现在你的输入流中，无任何前缀 | 老板对你下指令 | 直接在本屏幕响应，**禁止用** `tsh type` |
| `[TeamShell 消息] 来自 <名字>:` | 出现在你的输入流中，以此前缀开头 | Agent 同事发给你 | 用 `tsh type <对方的工号>` 回复 |
| 程序输出 | 不属于上述两种来源的所有输入 | 程序运行结果 | 正常处理 |

> **💡 为什么要严格区分**：这是协议的核心信任机制。如果分不清是谁在说话，协作就会混乱。前缀是身份的标识，就像人类社会的签名一样。

### 回复规则

收到带 `[TeamShell 消息]` 前缀的同事消息时：

- ✅ **必须回复**：消息里有问题、有任务、需要你行动 → 用 `tsh type` 回复
- ❌ **禁止回复**：消息是纯粹的结束语（"收到"、"好的"、"待命"、"没问题"、"结案"）→ 回复会导致无限循环，浪费 token
- ❌ **不应回复**：消息明显是错误发送的、或你确认对方不需要回应

收到老板的直接指令时（没有前缀）：
- 直接在本屏幕响应，**禁止用 `tsh type` 回复老板**

> **💡 沟通的黄金法则**：
> - 跟老板沟通：讲结果，不要讲过程
> - 跟同事沟通：讲清楚上下文和意图
> - 信息不足会导致协作效率低下，信息过载也会

---

## 任务监管

> **核心经验：谁派的活，谁负责盯到底。**
>
> 如果你给同事派了任务，你就是这个任务的**监管人**。
>
> 监管人不是发完消息就完事了，你要对任务的最终结果负责。

### 监管人的职责

| 职责 | 说明 |
|------|------|
| **进度监控** | 定期 `tsh status` 了解团队整体情况，`tsh query <工号>` 了解具体成员状态 |
| **障碍清除** | 发现 🟡缓慢 或 🔴静默 的成员，用 `tsh query` 了解详情，用 `tsh view` 深入查看 |
| **状态上报** | 如果障碍你也解决不了，汇总状态向老板汇报 |
| **结果验收** | 任务完成后检查交付质量，合格了再向老板闭环 |

> **💡 监控频率参考**：
> - 预计 < 1 分钟的任务：每 10 秒 `tsh status` 一次，完成后立即确认结果
> - 预计 1-10 分钟的任务：每 1-2 分钟 `tsh status` 一次
> - 预计 > 10 分钟的任务：每 5 分钟 `tsh status` 一次，🟡🔴 时用 `tsh query` 或 `tsh view` 深入查看

### 禁止事项

- ❌ **发完任务就忘了，再也不看了** → 任务十有八九会出问题
- ❌ **看到同事卡住了，假装没看见** → 互相帮衬是团队存在的价值
- ❌ **把问题直接甩给老板，不做任何解释** → 老板信息不足，做不出正确决策
- ❌ **任务失败了不汇报，老板问起来才说** → 及时汇报是你的责任

---

## tsh 命令手册

`tsh` 是 TeamShell 的命令行工具，用于管理工位和 Agent 间通信。

### 工位管理

| 命令 | 说明 |
|------|------|
| `tsh list` | 列出所有在职工位（工号 + 名字） |
| `tsh status` | 查看团队状态看板（活跃程度、任务、进度） |
| `tsh open "名字" -- claude` | 招募一个 Agent 工位 |
| `tsh open "服务" -- java -jar app.jar` | 启动一个程序工位 |
| `tsh open "构建" --cwd /path -- make build` | 指定工作目录启动 |
| `tsh open "助手" --env API_KEY=xxx -- node bot.js` | 注入环境变量 |
| `tsh open "构建" --auto-shell -- make build` | 自动用 shell 包装命令 |
| `tsh close <工号>` | 关闭工位，终止上面运行的程序 |
| `tsh name <工号> "新名字"` | 修改工位显示名称 |

### 工位间协作

| 命令 | 说明 |
|------|------|
| `tsh view <工号>` | 查看指定工位屏幕的最后 50 行（只读，对方无感知） |
| `tsh view <工号> 200` | 查看最后 200 行 |
| `tsh type <工号> "消息"` | 给指定工位发消息（自动回车） |
| `tsh type <工号> --no-enter "文本"` | 敲入文本，不自动回车 |
| `tsh type <工号> --key "\x03"` | 发送按键（如 Ctrl+C） |

### 状态汇报

| 命令 | 说明 |
|------|------|
| `tsh report task "重构登录模块"` | 设置当前任务描述 |
| `tsh report progress 60` | 报告进度，整数，范围 0-100 |
| `tsh report status running` | 状态：`running` / `idle` / `blocked` / `done` / `error` |
| `tsh report blocked "需要读取 /data/prod/ 的权限"` | 快捷方式 = status blocked + reason |
| `tsh report done` | 快捷方式 = status done + progress 100 |

### 成员查询

| 命令 | 说明 |
|------|------|
| `tsh query <工号>` | 查询指定工位全部状态 |
| `tsh status` | 查看团队状态看板（活跃程度、任务、进度） |

### 协议注入

新 Agent 入职前必须先注入协议，否则无法识别团队消息：

```bash
tsh init                      # 交互式：将协议写入 Agent 配置文件
tsh init --dry-run            # 预览变更，不写入
tsh init -y                   # 强制覆盖，不询问
tsh init --show               # 输出协议内容到 stdout
```

### 命令选择规则

| 你想做什么 | 使用命令 |
|-----------|---------|
| 了解团队整体情况 | `tsh status` |
| 了解某人的详细状态 | `tsh query <工号>` |
| 看某人具体在干什么 | `tsh view <工号>` |
| 跟某人说话/讨论/派活 | `tsh type <工号> "内容"` |
| 更新自己的进展 | `tsh report progress <数字>` |
| 开始新任务 | `tsh report task "任务描述"` |
| 自己卡住了 | `tsh report blocked "原因"` |
| 自己做完了 | `tsh report done` |
| 招募新成员 | `tsh open "名字" -- 命令` |
| 关闭某个工位 | `tsh close <工号>` |
| 初始化项目协议 | `tsh init` |

⚠️ **`tsh report` 写到共享看板，对方不会收到通知**
⚠️ **`tsh type` 直接发消息给对方，对方终端立刻显示。需要对方回应时用这个**

⚠️ `tsh type 1 "我做到60%了"` → 打断对方，浪费 token
✅ `tsh report progress 60` → 写到看板，不打扰任何人

⚠️ `tsh report "帮我查日志"` → report 是写状态，不是发消息
✅ `tsh type 2 "帮我查日志"` → 需要对方行动，用 type

---

## 垂直标签栏 + 信息面板

WezTeam 在 WezTerm 基础上增加了**垂直标签栏信息面板**，每个 Tab 下方显示上下文信息：

| 信息项 | 说明 | 示例 |
|--------|------|------|
| **路径** | 当前工作目录（倒序显示） | `wezteam\work\D:` |
| **Git 分支** | 当前 Git 分支名 | `git:main` |
| **当前命令** | 正在运行的进程 | `cargo build` |

配置示例：

```lua
config.tab_bar_vertical = true
config.tab_bar_vertical_width = 250
config.tab_bar_vertical_position = "Left"

config.colors = {
  tab_bar = {
    extra_info = {
      path = { show = true, fg_color = '#6699ff', intensity = 'Bold' },
      git_branch = { show = true, fg_color = '#50fa7b', italic = true },
      current_command = { show = true, fg_color = '#bd93f9' },
    },
  },
}
```

详细配置见 [Tab Bar Extra Info 文档](docs/config/lua/config/tab_bar_extra_info.md)。

---

## 安装构建

### 前置依赖

- [Rust](https://www.rust-lang.org/tools/install)（最新稳定版）
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: 参见 [上游构建文档](README.upstream.md)

### 从源码构建

```bash
git clone https://github.com/wsyb/wezteam.git
cd wezteam
cargo build --release --package wezterm-gui
```

### 打包安装程序

#### Windows (.exe 安装包)

前置依赖：[Inno Setup 6](https://jrsoftware.org/isdl.php)

```cmd
3-release.cmd                :: 一键构建+打包
1-build.cmd && 2-pkg-windows.cmd  :: 分步执行
```

#### Linux (.tar.gz)

```bash
cargo build -p wezterm --release -p wezterm-gui --release -p wezterm-mux-server --release -p strip-ansi-escapes --release
TAG_NAME=v0.1.0 bash 2-pkg-linux.sh
```

#### macOS (.zip App Bundle)

```bash
cargo build -p wezterm --release -p wezterm-gui --release -p wezterm-mux-server --release -p strip-ansi-escapes --release
TAG_NAME=v0.1.0 bash 2-pkg-macos.sh
```

> macOS 代码签名需配置 Apple Developer 证书。其他格式（deb/rpm/AppImage/Flatpak）见 `ci/` 目录。

---

## 配置说明

配置文件：Windows `%USERPROFILE%\.wezterm.lua`，macOS/Linux `~/.wezterm.lua`

### 垂直标签栏

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `tab_bar_vertical` | boolean | `true` | 是否使用垂直标签栏 |
| `tab_bar_vertical_width` | number | `250` | 垂直标签栏宽度（像素） |
| `tab_bar_vertical_position` | string | `"Left"` | 位置：`"Left"` 或 `"Right"` |

### 信息面板

通过 `config.colors.tab_bar.extra_info` 配置，每项支持：

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `show` | boolean | `true` | 是否显示 |
| `fg_color` | string | 调色板默认色 | 前景色 |
| `bg_color` | string | tab 背景 | 背景色 |
| `intensity` | string | `'Normal'` | `'Half'` / `'Normal'` / `'Bold'` |
| `italic` | boolean | `false` | 是否斜体 |
| `underline` | string | `'None'` | `'None'` / `'Single'` / `'Double'` / `'Curly'` / `'Dotted'` / `'Dashed'` |

---

## 关于本项目

WezTeam 是基于 [WezTerm](https://github.com/wezterm/wezterm)（由 [@wez](https://github.com/wez) 开发的 GPU 加速跨平台终端模拟器）的增强分支。

> **上游项目**：[wezterm/wezterm](https://github.com/wezterm/wezterm)
>
> 本项目遵循上游的 [MIT 许可证](LICENSE.md)，原始版权归属 Wez Furlong。

### 已知限制

- 进程启动/退出不触发事件，命令显示在标题变化/鼠标/焦点变化时更新
- OSC 7 CWD 某些场景可能不准确，此时优先使用 tab 标题路径

### 代码结构

| 模块 | 路径 | 说明 |
|------|------|------|
| 数据层 | `wezterm-gui/src/termwindow/tab_extra_info.rs` | 路径提取、Git 分支、进程信息 |
| UI 层 | `wezterm-gui/src/termwindow/render/fancy_tab_bar.rs` | 信息面板渲染 |
| 配置层 | `config/src/color.rs` | `ExtraInfoStyle` / `ExtraInfoItemStyle` |
| TeamShell CLI | `tsh/` | 工位管理命令行工具 |
| 协议文档 | `TeamShellProtocol.md` / `AGENTS.md` | 协作协议全文 |

## 许可证

[MIT License](LICENSE.md)

原始项目版权 (c) 2018-Present Wez Furlong
