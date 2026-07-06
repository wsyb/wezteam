# WezTeam - AI Agent Team Collaboration Terminal

[中文文档 →](README.zh-CN.md)

<p align="center">
  <strong>WezTerm + TeamShell</strong>
</p>

<p align="center">
  Build an AI team in your terminal. Assign tasks, observe colleagues, communicate, and deliver together.
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> ·
  <a href="#team-shell">TeamShell</a> ·
  <a href="#tsh-commands">tsh Commands</a> ·
  <a href="#message-handling">Message Handling</a> ·
  <a href="#task-supervision">Task Supervision</a> ·
  <a href="#vertical-tab-bar">Features</a> ·
  <a href="#installation">Installation</a> ·
  <a href="#configuration">Configuration</a>
</p>

---

## Quick Start

### Start Your First Agent (Recommended)

```bash
tsh open "My Assistant" -- claude
```

The backend automatically injects `TEAMSH_TAB_ID`, `TEAMSH_NAME`, and `TEAMSH_PLATFORM` environment variables.

### Initialize Project Protocol

```bash
tsh init              # Interactive: writes protocol to detected Agent config files
tsh init --dry-run    # Preview changes without writing
tsh init -y           # Force overwrite without prompting
tsh init --show       # Output protocol content to stdout
```

### Verify Team Status

```bash
tsh status    # View team overview
tsh list      # List all active workstations
```

---

## TeamShell — Multi-Agent Collaboration

WezTeam's core innovation is **TeamShell**: a protocol and toolchain that enables multiple AI Agents to collaborate within a single terminal window.

### Why TeamShell?

A single AI Agent has limited capabilities. For complex tasks, you need a **team**: frontend expert, backend expert, test engineer... TeamShell lets you run multiple Agents simultaneously in one terminal window. They can:

- **Observe each other** — View colleagues' screen output
- **Communicate** — Send messages, assign tasks
- **Coordinate** — Auto-divide work, monitor progress, report results
- **Scale dynamically** — Recruit new Agents as needed, dismiss when done

### How It Works

Each Agent runs in a **workstation** (terminal Tab). `tsh` (TeamShell CLI) manages workstations and Agent communication.

```
┌─────────────────────────────────────────────┐
│  Terminal Window (WezTeam)                   │
│                                              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ Slot 1   │ │ Slot 2   │ │ Slot 3   │     │
│  │ Xiaoming │ │ Xiaohong │ │ Logger   │     │
│  │ (Agent)  │ │ (Agent)  │ │ (Program)│     │
│  │          │ │          │ │          │     │
│  │ Writing  │ │ Testing  │ │ tail -f  │     │
│  │ Frontend │ │ APIs      │ │ app.log  │     │
│  └──────────┘ └──────────┘ └──────────┘     │
│                                              │
│  tsh list  →  1 Xiaoming  2 Xiaohong  3 Logger
│  tsh view 2  →  View Xiaohong's screen      │
│  tsh type 2 "Need help?"  →  Message Xiaohong
└─────────────────────────────────────────────┘
```

### Two Types of Workstations

| Type | Description | Interaction |
|------|-------------|-------------|
| **Agent Workstation** (Live) | Runs LLM with TeamShell protocol | Recognizes messages,主动协作, replies to colleagues |
| **Program Workstation** (Static) | Runs regular programs (Java/Vim/Shell) | Controlled via stdin only, no主动响应 |

> **How to identify** (经验参考): Workstations with human-like names (e.g. "Frontend Expert") are usually Live; service-like names (e.g. "Log Service") are usually Static. Use `tsh view` to confirm if needed.

### Protocol Core Principles

1. **Identity Verification** — Agent startup verified via `TEAMSH_TAB_ID` + `tsh list`, ensuring正式团队成员
2. **Message Identification** — `[TeamShell Message] from <name>:` prefix distinguishes colleague messages from program output
3. **Autonomous Decision Making** — Agents make judgments, no need for step-by-step approval, but irreversible operations require asking the boss first
4. **Task Supervision** — Whoever assigns the task monitors until completion, issues escalated layer by layer

Full protocol: [TeamShellProtocol.md](TeamShellProtocol.md) | [AGENTS.md](AGENTS.md)

---

## Message Handling

### How to Identify Message Sources

| Source | Identification | Meaning | How to Respond |
|--------|---------------|---------|---------------|
| User input | No prefix in input stream | Boss giving you instructions | Respond directly, **never** use `tsh type` |
| `[TeamShell Message] from <name>:` | Starts with this prefix | Agent colleague messaging you | Reply using `tsh type <their ID>` |
| Program output | Doesn't match above | Program running results | Handle normally |

> **Why strict identification matters**: This is the core trust mechanism of the protocol. If you can't tell who's speaking, collaboration breaks down. The prefix is like a signature in human society.

### Reply Rules

When receiving a message with `[TeamShell Message]` prefix:

- ✅ **Must reply**: Message has questions, tasks, or needs your action → use `tsh type` to reply
- ❌ **Forbidden to reply**: Message is pure closing ("Received", "OK", "Standing by", "No problem", "Case closed") → replying causes infinite loops
- ❌ **Should not reply**: Message is obviously mis-sent, or you confirm they don't need a response

When receiving boss's direct instructions (no prefix):
- Respond directly in your current screen, **forbidden to use `tsh type`**

> **Golden Rule of Communication**:
> - With boss: Share results, not process
> - With colleagues: Share context and intent
> - Insufficient information causes inefficiency; too much causes overload

---

## Task Supervision

> **Core Principle: Whoever assigns the task, monitors until completion.**

If you assigned a task to a colleague, you are the **supervisor** of that task. The supervisor is not done after sending the message — you are responsible for the final result.

### Supervisor Responsibilities

| Responsibility | Description |
|----------------|-------------|
| **Progress Monitoring** | Regularly run `tsh status` for team overview, `tsh query <ID>` for specific member status |
| **Obstacle Clearing** | When discovering 🟡 slow or 🔴 silent members, use `tsh query` for details, `tsh view` for deep inspection |
| **Status Reporting** | If you can't solve the obstacle, summarize status and report to boss |
| **Result Acceptance** | After task completion, check delivery quality before closing with boss |

> **Monitoring Frequency Guide**:
> - Estimated < 1 min task: `tsh status` every 10 seconds, confirm result immediately after completion
> - Estimated 1-10 min task: `tsh status` every 1-2 minutes
> - Estimated > 10 min task: `tsh status` every 5 minutes, use `tsh query` or `tsh view` when 🟡🔴

### Forbidden Actions

- ❌ **Assign and forget** → Task will likely fail
- ❌ **Ignore colleagues stuck** → Mutual help is the value of having a team
- ❌ **Dump problems on boss without explanation** → Boss lacks context to make correct decisions
- ❌ **Hide task failure, only report when boss asks** → Timely reporting is your responsibility

---

## tsh Commands

`tsh` is the TeamShell CLI for managing workstations and Agent communication.

### Workstation Management

| Command | Description |
|---------|-------------|
| `tsh list` | List all active workstations (ID + Name) |
| `tsh status` | View team status board (activity, tasks, progress) |
| `tsh open "Name" -- claude` | Recruit an Agent workstation |
| `tsh open "Service" -- java -jar app.jar` | Start a program workstation |
| `tsh open "Builder" --cwd /path -- make build` | Start with working directory |
| `tsh open "Helper" --env KEY=val -- node bot.js` | Inject environment variables |
| `tsh open "Auto" --auto-shell -- make build` | Auto-wrap with shell |
| `tsh close <ID>` | Close workstation, terminate running program |
| `tsh name <ID> "New Name"` | Rename workstation display name |

### Inter-Workstation Collaboration

| Command | Description |
|---------|-------------|
| `tsh view <ID>` | View last 50 lines of workstation screen (read-only, invisible to对方) |
| `tsh view <ID> 200` | View last 200 lines |
| `tsh type <ID> "message"` | Send message to workstation (auto-enter) |
| `tsh type <ID> --no-enter "text"` | Type text without auto-enter |
| `tsh type <ID> --key "\x03"` | Send keystroke (e.g. Ctrl+C) |

### Status Reporting

| Command | Description |
|---------|-------------|
| `tsh report task "Description"` | Set current task description |
| `tsh report progress 60` | Report progress (0-100) |
| `tsh report status running` | Status: `running` / `idle` / `blocked` / `done` / `error` |
| `tsh report blocked "Reason"` | Shortcut: status blocked + reason |
| `tsh report done` | Shortcut: status done + progress 100 |

### Member Query

| Command | Description |
|---------|-------------|
| `tsh query <ID>` | Query detailed status of specific member |
| `tsh status` | View team overview (activity, tasks, progress) |

### Protocol Injection

```bash
tsh init                      # Interactive: writes protocol to Agent config files
tsh init --dry-run            # Preview changes without writing
tsh init -y                   # Force overwrite without prompting
tsh init --show               # Output protocol to stdout
```

New Agents must inject the protocol before joining, otherwise they cannot recognize team messages.

### Command Selection Guide

| What you want to do | Use this command |
|-------------------|-----------------|
| View team overview | `tsh status` |
| View specific member details | `tsh query <ID>` |
| See what someone is doing | `tsh view <ID>` |
| Talk / assign tasks | `tsh type <ID> "content"` |
| Update your progress | `tsh report progress <number>` |
| Start new task | `tsh report task "description"` |
| You're stuck | `tsh report blocked "reason"` |
| You're done | `tsh report done` |
| Recruit new member | `tsh open "name" -- command` |
| Close a workstation | `tsh close <ID>` |
| Initialize project protocol | `tsh init` |

⚠️ `tsh report` writes to shared board —对方不会收到通知
⚠️ `tsh type` sends message directly — 对方终端立刻显示。需要对方回应时用这个

⚠️ `tsh type 1 "I'm at 60%"` → Interrupts对方, wastes token
✅ `tsh report progress 60` → Writes to board, no打扰

⚠️ `tsh report "Help me check logs"` → report is for status, not messaging
✅ `tsh type 2 "Help me check logs"` → Need对方to act, use type

---

## Vertical Tab Bar + Info Panel

WezTeam adds a **vertical tab bar info panel** based on WezTerm, displaying context info below each Tab:

| Info Type | Description | Example |
|-----------|-------------|---------|
| **Path** | Current working directory (reversed) | `wezteam\work\D:` |
| **Git Branch** | Current Git branch name | `git:main` |
| **Current Command** | Running process | `cargo build` |

Configuration example:

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

See [Tab Bar Extra Info Docs](docs/config/lua/config/tab_bar_extra_info.md) for details.

---

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: See [upstream build docs](README.upstream.md)

### Build from Source

```bash
git clone https://github.com/wsyb/wezteam.git
cd wezteam
cargo build --release --package wezterm-gui
```

### Package Installer

#### Windows (.exe installer)

Prerequisite: [Inno Setup 6](https://jrsoftware.org/isdl.php)

```cmd
3-release.cmd                :: One-click build + package
1-build.cmd && 2-pkg-windows.cmd  :: Step by step
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

> macOS code signing requires Apple Developer certificate. Other formats (deb/rpm/AppImage/Flatpak) see `ci/` directory.

---

## Configuration

Config file: Windows `%USERPROFILE%\.wezterm.lua`, macOS/Linux `~/.wezterm.lua`

### Vertical Tab Bar

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `tab_bar_vertical` | boolean | `true` | Enable vertical tab bar |
| `tab_bar_vertical_width` | number | `250` | Vertical tab bar width (pixels) |
| `tab_bar_vertical_position` | string | `"Left"` | Position: `"Left"` or `"Right"` |

### Info Panel

Configured via `config.colors.tab_bar.extra_info`, each item supports:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `show` | boolean | `true` | Whether to display |
| `fg_color` | string | Palette default | Foreground color |
| `bg_color` | string | Tab background | Background color |
| `intensity` | string | `'Normal'` | `'Half'` / `'Normal'` / `'Bold'` |
| `italic` | boolean | `false` | Whether italic |
| `underline` | string | `'None'` | `'None'` / `'Single'` / `'Double'` / `'Curly'` / `'Dotted'` / `'Dashed'` |

---

## About This Project

WezTeam is an enhanced fork of [WezTerm](https://github.com/wezterm/wezterm) (GPU-accelerated cross-platform terminal emulator by [@wez](https://github.com/wez)).

> **Upstream**: [wezterm/wezterm](https://github.com/wezterm/wezterm)
>
> This project follows upstream's [MIT License](LICENSE.md). Original copyright belongs to Wez Furlong.

### Known Limitations

- Process start/exit does not trigger events; command display updates on title change/mouse/focus change
- OSC 7 CWD may be inaccurate in some scenarios; prefer tab title path in such cases

### Code Structure

| Module | Path | Description |
|--------|------|-------------|
| Data Layer | `wezterm-gui/src/termwindow/tab_extra_info.rs` | Path extraction, Git branch, process info |
| UI Layer | `wezterm-gui/src/termwindow/render/fancy_tab_bar.rs` | Info panel rendering |
| Config Layer | `config/src/color.rs` | `ExtraInfoStyle` / `ExtraInfoItemStyle` |
| TeamShell CLI | `tsh/` | Workstation management CLI |
| Protocol Docs | `TeamShellProtocol.md` / `AGENTS.md` | Full collaboration protocol |

## License

[MIT License](LICENSE.md)

Original project copyright (c) 2018-Present Wez Furlong
