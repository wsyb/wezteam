# WezTeam

<p align="center">
  <strong>AI Agent 团队协作终端</strong>
</p>

<p align="center">
  基于 WezTerm · 内置 TeamShell 协作协议 · 多 Agent 工位管理
</p>

<p align="center">
  <a href="#teamshell--多-agent-协作">TeamShell</a> ·
  <a href="#tsh-命令手册">tsh 命令</a> ·
  <a href="#垂直标签栏--信息面板">功能特性</a> ·
  <a href="#安装构建">安装构建</a> ·
  <a href="#配置说明">配置</a>
</p>

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

## tsh 命令手册

`tsh` 是 TeamShell 的命令行工具，用于管理工位和 Agent 间通信。

### 工位管理

| 命令 | 说明 |
|------|------|
| `tsh list` | 列出所有在职工位（工号 + 名字） |
| `tsh open "名字" -- claude` | 招募一个 Agent 工位 |
| `tsh open "服务" -- java -jar app.jar` | 启动一个程序工位 |
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

### 协议注入

新 Agent 入职前必须先注入协议，否则无法识别团队消息：

```bash
tsh init AGENTS.md        # 注入协议
tsh open "小明" -- claude  # 然后启动
```

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
