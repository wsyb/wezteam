# WezTeam - AI Agent Team Collaboration Terminal

[中文文档 →](README.zh-CN.md)

<p align="center">
  <strong>WezTerm + TeamShell</strong>
</p>

<p align="center">
  Build an AI team in your terminal. Assign tasks, observe colleagues, communicate, and deliver together.
</p>

<p align="center">
  <a href="#team-shell-multi-agent-collaboration">TeamShell</a> ·
  <a href="#tsh-command-reference">tsh Commands</a> ·
  <a href="#vertical-tab-bar-info-panel">Features</a> ·
  <a href="#installation">Installation</a> ·
  <a href="#configuration">Configuration</a>
</p>

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

### Protocol Core Principles

1. **Identity Verification** — Agent startup verified via `TEAMSH_TAB_ID` + `tsh list`, ensuring正式团队成员
2. **Message Identification** — `[TeamShell Message] from <name>:` prefix distinguishes colleague messages from program output
3. **Autonomous Decision Making** — Agents make judgments, no need for step-by-step approval, but irreversible operations require asking the boss first
4. **Task Supervision** — Whoever assigns the task monitors until completion, issues escalated layer by layer

Full protocol: [TeamShellProtocol.md](TeamShellProtocol.md) | [AGENTS.md](AGENTS.md)

---

## tsh Command Reference

`tsh` is the TeamShell CLI for managing workstations and Agent communication.

### Workstation Management

| Command | Description |
|---------|-------------|
| `tsh list` | List all active workstations (ID + Name) |
| `tsh open "Name" -- claude` | Recruit an Agent workstation |
| `tsh open "Service" -- java -jar app.jar` | Start a program workstation |
| `tsh close <ID>` | Close workstation, terminate running program |
| `tsh name <ID> "New Name"` | Rename workstation display name |

### Inter-Workstation Collaboration

| Command | Description |
|---------|-------------|
| `tsh view <ID>` | View last 50 lines of workstation screen (read-only) |
| `tsh view <ID> 200` | View last 200 lines |
| `tsh type <ID> "message"` | Send message to workstation (auto-enter) |
| `tsh type <ID> --no-enter "text"` | Type text without auto-enter |
| `tsh type <ID> --key "\x03"` | Send keystroke (e.g. Ctrl+C) |

### Protocol Injection

New Agents must inject the protocol before joining, otherwise they cannot recognize team messages:

```bash
tsh init AGENTS.md        # Inject protocol
tsh open "Xiaoming" -- claude  # Then start
```

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
