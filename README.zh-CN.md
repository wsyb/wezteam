# WezTeam - AI Agent 团队协作终端

[English Documentation →](README.md)

<p align="center">
  <strong>WezTerm + TeamShell</strong>
</p>

<p align="center">
  在终端标签页中运行 AI Agent。分配任务、观察同事、沟通协作、共同交付。
</p>

---

## 🚀 快速开始（面向最终用户）

**如果你只是想使用 WezTeam，请直接跳到 [下载安装](#下载安装)。**

### 下载安装

#### Windows

1. 访问 [Releases](https://github.com/wsyb/wezteam/releases/latest)
2. 下载 `WezTerm-*-setup.exe`
3. 双击安装
4. 从开始菜单启动

#### macOS

1. 访问 [Releases](https://github.com/wsyb/wezteam/releases/latest)
2. 下载 `WezTerm-*-macos.zip`
3. 解压并将 `WezTerm.app` 拖到 `/Applications`
4. 从 Launchpad 或 Spotlight 启动

#### Linux

**Ubuntu/Debian**:
```bash
# 下载并安装
wget https://github.com/wsyb/wezteam/releases/latest/download/wezterm-*-debian*.tar.xz
tar -xf wezterm-*.tar.xz
cd wezterm-*
sudo dpkg -i .
```

**Fedora/CentOS**:
```bash
wget https://github.com/wsyb/wezteam/releases/latest/download/wezterm-*.rpm
sudo dnf install wezterm-*.rpm  # Fedora
# sudo yum install wezterm-*.rpm  # CentOS
```

**AppImage**（通用版，适用于所有 Linux）:
```bash
wget https://github.com/wsyb/wezteam/releases/latest/download/WezTerm-*.AppImage
chmod +x WezTerm-*.AppImage
./WezTerm-*.AppImage
```

### 启动你的第一个 AI Agent

打开 WezTeam 终端，在任意标签页运行：

```bash
tsh open "我的助手" -- claude
```

就这么简单。Claude 会在新标签页中启动并加入你的团队。

### 验证是否成功

```bash
tsh status
```

**预期输出**:
```
1号(我的助手)  🟢 活跃  最后活动: 刚刚   任务: 待分配  进度: 0%
```

### 尝试团队协作

```bash
tsh type 1 "你好！你能做什么？"   # 发送消息
tsh view 1                         # 查看屏幕
```

---

## 🤖 什么是 TeamShell？

**TeamShell** 是 WezTeam 的核心创新：一套协议和 CLI 工具（`tsh`），让多个 AI Agent 在终端标签页中协作。

### 为什么需要 TeamShell？

单个 AI Agent 能力有限。复杂任务需要**团队**：前端专家、后端专家、测试工程师……

TeamShell 让你同时运行多个 Agent，它们可以：
- **互相观察** — 查看同事的屏幕输出
- **互相通信** — 发送消息、分配任务
- **互相协调** — 自动分工、监管进度、汇报结果
- **动态扩编** — 按需招募新 Agent，任务完成后解散

### 工作原理

每个 Agent 运行在**工位**（终端 Tab）中。`tsh` 管理工位和 Agent 通信。

```
┌─────────────────────────────────────────────┐
│  WezTeam 终端窗口                            │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ 标签页 1 │ │ 标签页 2 │ │ 标签页 3 │     │
│  │ 我的助手 │ │ 前端专家 │ │ 日志服务 │     │
│  │ (Agent)  │ │ (Agent)  │ │ (程序)   │     │
│  │ 正在写   │ │ 正在测试 │ │ tail -f  │     │
│  │ 前端代码 │ │ API      │ │ app.log  │     │
│  └──────────┘ └──────────┘ └──────────┘     │
└─────────────────────────────────────────────┘
```

### 两种工位类型

| 类型 | 说明 | 交互方式 |
|------|------|---------|
| **Agent 工位** | 运行带 TeamShell 协议的 LLM | 能识别消息、主动协作、回复同事 |
| **程序工位** | 运行普通程序（Java/Vim/Shell） | 只能通过标准输入控制，不会主动响应 |

---

## 📖 核心命令

### 工位管理

| 命令 | 说明 |
|------|------|
| `tsh list` | 列出所有在职工位（工号 + 名字） |
| `tsh status` | 查看团队状态看板（活跃程度、任务、进度） |
| `tsh open "名字" -- claude` | 招募一个 Agent 工位 |
| `tsh open "服务" -- java -jar app.jar` | 启动一个程序工位 |
| `tsh close <工号>` | 关闭工位，终止上面运行的程序 |

### 工位间协作

| 命令 | 说明 |
|------|------|
| `tsh view <工号>` | 查看工位屏幕最后 50 行（只读，对方无感知） |
| `tsh view <工号> 200` | 查看最后 200 行 |
| `tsh type <工号> "消息"` | 发送消息（自动回车） |
| `tsh type <工号> --no-enter "文本"` | 敲入文本，不自动回车 |

### 状态汇报

| 命令 | 说明 |
|------|------|
| `tsh report task "任务描述"` | 设置当前任务 |
| `tsh report progress 60` | 报告进度（0-100） |
| `tsh report blocked "原因"` | 汇报遇到阻塞 |
| `tsh report done` | 标记任务完成 |

### 快速决策指南

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

> ⚠️ **`tsh report`** 写到共享看板，对方不会收到通知
> ⚠️ **`tsh type`** 直接发消息给对方，对方终端立刻显示。需要对方回应时用这个

⚠️ `tsh type 1 "我做到60%了"` → 打断对方，浪费 token
✅ `tsh report progress 60` → 写到看板，不打扰任何人

⚠️ `tsh report "帮我查日志"` → report 是写状态，不是发消息
✅ `tsh type 2 "帮我查日志"` → 需要对方行动，用 type

⚠️ `tsh name 2 "前端专家-做60%"` → 名字不是状态栏
✅ `tsh report progress 60` → 状态用 report

---

## ⚙️ 安装

### 从安装包安装（推荐用户使用）

见上方 [下载安装](#下载安装) 章节。

### 从源码构建（面向开发者）

**前置条件**：
- [Rust](https://www.rust-lang.org/tools/install)（最新稳定版）
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: 参见 [上游构建文档](README.upstream.md)

**构建**：
```bash
git clone https://github.com/wsyb/wezteam.git
cd wezteam
cargo build --release --package wezterm-gui
```

**打包安装程序**：
- Windows: `3-release.cmd`（一键构建+打包）或 `1-build.cmd && 2-pkg-windows.cmd`（分步执行）
- Linux: `TAG_NAME=v0.1.0 bash 2-pkg-linux.sh`
- macOS: `TAG_NAME=v0.1.0 bash 2-pkg-macos.sh`

---

## 🎨 功能特性

### 垂直标签栏 + 信息面板

WezTeam 在 WezTerm 基础上增加了**垂直标签栏信息面板**，每个 Tab 下方显示上下文信息：

| 信息项 | 说明 | 示例 |
|--------|------|------|
| **路径** | 当前工作目录（倒序显示） | `wezteam\work\D:` |
| **Git 分支** | 当前 Git 分支名 | `git:main` |
| **当前命令** | 正在运行的进程 | `cargo build` |

**配置示例**（`~/.wezterm.lua` 或 `%USERPROFILE%\.wezterm.lua`）：

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

## 🧭 TeamShell 协议

### 消息处理

**怎么识别消息来源**：

| 来源 | 判定方法 | 响应方式 |
|------|---------|---------|
| 用户直接输入 | 无任何前缀 | 直接在本屏幕响应，**禁止用** `tsh type` |
| `[TeamShell 消息] 来自 <名字>:` | 以此前缀开头 | 用 `tsh type <对方的工号>` 回复 |
| 程序输出 | 不属于上述两种 | 正常处理 |

**回复规则**：
- ✅ **必须回复**：消息有问题、任务、需要行动 → 用 `tsh type`
- ❌ **禁止回复**：纯粹结束语（"收到"、"好的"、"待命"）→ 导致无限循环
- ❌ **不应回复**：明显错误发送或确认不需要回应

**沟通的黄金法则**：
- 跟老板沟通：讲结果，不要讲过程
- 跟同事沟通：讲清楚上下文和意图

### 任务监管

> **核心经验：谁派的活，谁负责盯到底。**

如果你给同事派了任务，你就是这个任务的**监管人**。

| 职责 | 说明 |
|------|------|
| **进度监控** | 定期 `tsh status` 了解团队整体情况 |
| **障碍清除** | 发现缓慢/静默成员，用 `tsh query` 了解详情 |
| **状态上报** | 障碍无法解决时，汇总状态向老板汇报 |
| **结果验收** | 检查交付质量，合格后再闭环 |

完整协议见 [TeamShellProtocol.md](TeamShellProtocol.md) 和 [AGENTS.md](AGENTS.md)。

---

## 🛠️ tsh 命令手册

### 工位管理

| 命令 | 说明 |
|------|------|
| `tsh list` | 列出所有在职工位（工号 + 名字） |
| `tsh status` | 查看团队状态看板 |
| `tsh query <工号>` | 查询指定工位详细状态 |
| `tsh open "名字" -- claude` | 招募一个 Agent 工位 |
| `tsh open "构建" --cwd /path -- make build` | 指定工作目录启动 |
| `tsh open "助手" --env API_KEY=xxx -- node bot.js` | 注入环境变量 |
| `tsh open "构建" --auto-shell -- make build` | 自动用 shell 包装命令 |
| `tsh open "助手" --init-prompt "你好" -- claude` | 创建后自动发送入职消息 |
| `tsh close <工号>` | 关闭工位，终止上面运行的程序 |
| `tsh name <工号> "新名字"` | 修改工位显示名称 |

### 工位间协作

| 命令 | 说明 |
|------|------|
| `tsh view <工号>` | 查看指定工位屏幕最后 50 行（只读） |
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
| `tsh report blocked "需要权限"` | 快捷方式 = status blocked + reason |
| `tsh report done` | 快捷方式 = status done + progress 100 |

### 协议注入

新 Agent 入职前必须先注入协议：

```bash
tsh init                      # 交互式写入协议到 Agent 配置文件
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

⚠️ `tsh name 2 "前端专家-做60%"` → 名字不是状态栏
✅ `tsh report progress 60` → 状态用 report

---

## 💻 面向开发者（从源码构建）

如果你想**贡献代码**或**修改 WezTeam**，请参考 [BUILD.md](BUILD.md)（英文），里面有详细的构建说明、前置条件和故障排除。

**快速概览**：

```bash
git clone https://github.com/wsyb/wezteam.git
cd wezteam
cargo build --release --package wezterm-gui
```

---

## 📚 更多资料

- 📖 [TeamShell 协议](TeamShellProtocol.md) — 完整协作协议
- 📖 [AGENTS.md](AGENTS.md) — Agent 配置指南
- 🎨 [标签栏配置](docs/config/lua/config/tab_bar_extra_info.md) — 垂直标签栏自定义
- 📦 [Releases](https://github.com/wsyb/wezteam/releases) — 下载最新版本

---

## 关于本项目

WezTeam 是基于 [WezTerm](https://github.com/wezterm/wezterm)（由 [@wez](https://github.com/wez) 开发的 GPU 加速跨平台终端模拟器）的增强分支。

> **上游项目**：[wezterm/wezterm](https://github.com/wezterm/wezterm)
>
> 本项目遵循上游的 [MIT 许可证](LICENSE.md)。

### 已知限制

- 进程启动/退出不触发事件，命令显示在标题变化/鼠标/焦点变化时更新
- OSC 7 CWD 某些场景可能不准确，此时优先使用 tab 标题路径

### 代码结构

| 模块 | 路径 | 说明 |
|------|------|------|
| 数据层 | `wezterm-gui/src/termwindow/tab_extra_info.rs` | 路径、Git 分支、进程信息 |
| UI 层 | `wezterm-gui/src/termwindow/render/fancy_tab_bar.rs` | 信息面板渲染 |
| 配置层 | `config/src/color.rs` | `ExtraInfoStyle` / `ExtraInfoItemStyle` |
| TeamShell CLI | `tsh/` | 工位管理命令行工具 |
| 协议文档 | `TeamShellProtocol.md` / `AGENTS.md` | 协作协议全文 |

## 许可证

[MIT License](LICENSE.md)

原始项目版权 (c) 2018-Present Wez Furlong
