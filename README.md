# WezTeam - AI Agent Team Collaboration Terminal

[中文文档 →](README.zh-CN.md)

<p align="center">
  <strong>WezTerm + TeamShell</strong>
</p>

<p align="center">
  Run AI agents in terminal tabs. Assign tasks, observe colleagues, communicate, and ship together.
</p>

---

## 🚀 Quick Start (For End Users)

**Skip to [Download](#download) if you just want to use WezTeam.**

### Download

#### Windows

1. Go to [Releases](https://github.com/wsyb/wezteam/releases/latest)
2. Download `WezTerm-*-setup.exe`
3. Double-click to install
4. Launch from Start Menu

#### macOS

1. Go to [Releases](https://github.com/wsyb/wezteam/releases/latest)
2. Download `WezTerm-*-macos.zip`
3. Extract and drag `WezTerm.app` to `/Applications`
4. Launch from Launchpad or Spotlight

#### Linux

**Ubuntu/Debian**:
```bash
# Download and install
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

**AppImage** (Universal, works on any Linux):
```bash
wget https://github.com/wsyb/wezteam/releases/latest/download/WezTerm-*.AppImage
chmod +x WezTerm-*.AppImage
./WezTerm-*.AppImage
```

### Launch Your First AI Agent

Open WezTeam terminal, then run in any tab:

```bash
tsh open "My Assistant" -- claude
```

That's it. Claude starts in a new tab and joins your team.

### Verify It Works

```bash
tsh status
```

**Expected output**:
```
1号(My Assistant)  🟢 活跃  最后活动: 刚刚   任务: 待分配  进度: 0%
```

### Try Collaboration

```bash
tsh type 1 "Hello! What can you do?"   # Send a message
tsh view 1                              # View their screen
```

---

## 🤖 What Is TeamShell?

**TeamShell** is WezTeam's core innovation: a protocol and CLI (`tsh`) that lets multiple AI Agents collaborate in terminal tabs.

### Why TeamShell?

A single AI Agent has limitations. For complex tasks, you need a **team**: frontend expert, backend expert, test engineer...

TeamShell lets you run multiple Agents simultaneously. They can:
- **Observe each other** — View colleagues' screen output
- **Communicate** — Send messages, assign tasks
- **Coordinate** — Auto-divide work, monitor progress, report results
- **Scale dynamically** — Recruit new Agents as needed, dismiss when done

### How It Works

Each Agent runs in a **workstation** (terminal Tab). `tsh` manages workstations and Agent communication.

```
┌─────────────────────────────────────────────┐
│  WezTeam Terminal Window                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │ Tab 1    │ │ Tab 2    │ │ Tab 3    │     │
│  │ Assistant│ │ Frontend │ │ Logger   │     │
│  │ (Agent)  │ │ (Agent)  │ │ (Program)│     │
│  │ Writing  │ │ Testing  │ │ tail -f  │     │
│  │ Frontend │ │ APIs      │ │ app.log  │     │
│  └──────────┘ └──────────┘ └──────────┘     │
└─────────────────────────────────────────────┘
```

### Two Types of Workstations

| Type | Description | Interaction |
|------|-------------|-------------|
| **Agent Workstation** | Runs LLM with TeamShell protocol | Recognizes messages,主动协作, replies to colleagues |
| **Program Workstation** | Runs regular programs (Java/Vim/Shell) | Controlled via stdin only, no主动响应 |

---

## 📖 Essential Commands

### Workstation Management

| Command | Description |
|---------|-------------|
| `tsh list` | List all active workstations (ID + Name) |
| `tsh status` | View team status board (activity, tasks, progress) |
| `tsh open "Name" -- claude` | Recruit an Agent workstation |
| `tsh open "Service" -- java -jar app.jar` | Start a program workstation |
| `tsh close <ID>` | Close workstation, terminate running program |

### Inter-Workstation Collaboration

| Command | Description |
|---------|-------------|
| `tsh view <ID>` | View last 50 lines of workstation screen (read-only) |
| `tsh view <ID> 200` | View last 200 lines |
| `tsh type <ID> "message"` | Send message to workstation (auto-enter) |
| `tsh type <ID> --no-enter "text"` | Type text without auto-enter |

### Status Reporting

| Command | Description |
|---------|-------------|
| `tsh report task "Description"` | Set current task description |
| `tsh report progress 60` | Report progress (0-100) |
| `tsh report blocked "Reason"` | Report you're stuck |
| `tsh report done` | Mark task as complete |

### Quick Decision Guide

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

> ⚠️ **`tsh report`** writes to shared board — recipient won't be notified
> ⚠️ **`tsh type`** sends message directly — their terminal shows it immediately

**Common mistakes**:
- ❌ `tsh type 1 "I'm at 60%"` → Interrupts recipient, wastes token
- ✅ `tsh report progress 60` → Writes to board, no interruption

For complete command reference, see [tsh Commands](#tsh-commands) and [TeamShell Protocol](TeamShellProtocol.md).

---

## ⚙️ Installation

### From Installer (Recommended for Users)

See [Download](#download) section above.

### From Source (For Developers)

**Prerequisites**:
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: See [upstream build docs](README.upstream.md)

**Build**:
```bash
git clone https://github.com/wsyb/wezteam.git
cd wezteam
cargo build --release --package wezterm-gui
```

**Package Installer**:
- Windows: `3-release.cmd` (one-click) or `1-build.cmd && 2-pkg-windows.cmd` (step-by-step)
- Linux: `TAG_NAME=v0.1.0 bash 2-pkg-linux.sh`
- macOS: `TAG_NAME=v0.1.0 bash 2-pkg-macos.sh`

---

## 🎨 Features

### Vertical Tab Bar + Info Panel

WezTeam adds a **vertical tab bar info panel** based on WezTerm, displaying context info below each Tab:

| Info Type | Description | Example |
|-----------|-------------|---------|
| **Path** | Current working directory (reversed) | `wezteam\work\D:` |
| **Git Branch** | Current Git branch name | `git:main` |
| **Current Command** | Running process | `cargo build` |

**Configuration** (`~/.wezterm.lua` or `%USERPROFILE%\.wezterm.lua`):

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

For detailed configuration options, see [Tab Bar Extra Info Docs](docs/config/lua/config/tab_bar_extra_info.md).

---

## 🧭 TeamShell Protocol

### Message Handling

**How to identify message sources**:

| Source | Identification | How to Respond |
|--------|---------------|----------------|
| User input | No prefix | Respond directly, **never** use `tsh type` |
| `[TeamShell Message] from <name>:` | Starts with this prefix | Reply using `tsh type <their ID>` |
| Program output | Doesn't match above | Handle normally |

**Reply rules**:
- ✅ **Must reply**: Message has questions, tasks, or needs your action → use `tsh type`
- ❌ **Forbidden to reply**: Message is pure closing ("Received", "OK", "Standing by") → causes infinite loops
- ❌ **Should not reply**: Obviously mis-sent, or you confirm they don't need a response

### Task Supervision

> **Core Principle: Whoever assigns the task, monitors until completion.**

If you assigned a task to a colleague, you are the **supervisor**.

| Responsibility | Description |
|----------------|-------------|
| **Progress Monitoring** | Regularly run `tsh status` for team overview |
| **Obstacle Clearing** | When discovering slow/silent members, investigate and help |
| **Status Reporting** | If you can't solve it, summarize and report to boss |
| **Result Acceptance** | Check delivery quality before closing |

For complete protocol, see [TeamShellProtocol.md](TeamShellProtocol.md) and [AGENTS.md](AGENTS.md).

---

## 🛠️ Full tsh Commands

### Workstation Management

| Command | Description |
|---------|-------------|
| `tsh list` | List all active workstations (ID + Name) |
| `tsh status` | View team status board |
| `tsh query <ID>` | Query detailed status of specific member |
| `tsh open "Name" -- claude` | Recruit an Agent workstation |
| `tsh open "Builder" --cwd /path -- make build` | Start with working directory |
| `tsh open "Helper" --env KEY=val -- node bot.js` | Inject environment variables |
| `tsh open "Auto" --auto-shell -- make build` | Auto-wrap with shell |
| `tsh open "Init" --init-prompt "Hello" -- claude` | Auto-send welcome message after 3s |
| `tsh close <ID>` | Close workstation |
| `tsh name <ID> "New Name"` | Rename workstation |

### Inter-Workstation Collaboration

| Command | Description |
|---------|-------------|
| `tsh view <ID>` | View last 50 lines of screen (read-only) |
| `tsh view <ID> 200` | View last 200 lines |
| `tsh type <ID> "message"` | Send message (auto-enter) |
| `tsh type <ID> --no-enter "text"` | Type without auto-enter |
| `tsh type <ID> --key "\x03"` | Send keystroke (e.g. Ctrl+C) |

### Status Reporting

| Command | Description |
|---------|-------------|
| `tsh report task "Description"` | Set current task |
| `tsh report progress 60` | Report progress (0-100) |
| `tsh report status running` | Set status: `running` / `idle` / `blocked` / `done` / `error` |
| `tsh report blocked "Reason"` | Shortcut: blocked + reason |
| `tsh report done` | Shortcut: done + progress 100 |

### Protocol Injection

```bash
tsh init                      # Write protocol to Agent config files
tsh init --dry-run            # Preview without writing
tsh init -y                   # Force overwrite
tsh init --show               # Output to stdout
```

---

## 💻 Building from Source

**This section is for developers who want to contribute to WezTeam.**

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Windows: Visual Studio Build Tools
- macOS: Xcode Command Line Tools
- Linux: See [upstream build docs](README.upstream.md)

### Build Steps

```bash
# Clone the repository
git clone https://github.com/wsyb/wezteam.git
cd wezteam

# Build (5-10 minutes on first run)
cargo build --release --package wezterm-gui

# Run
cargo run --release --package wezterm-gui

# Or use the binary directly
./target/release/wezterm-gui      # macOS/Linux
./target/release/wezterm-gui.exe  # Windows
```

### Package Installer

- Windows: `3-release.cmd` (one-click build + package)
- Linux: `TAG_NAME=v0.1.0 bash 2-pkg-linux.sh`
- macOS: `TAG_NAME=v0.1.0 bash 2-pkg-macos.sh`

---

## 📚 Learn More

- 📖 [TeamShell Protocol](TeamShellProtocol.md) — Complete collaboration protocol
- 📖 [AGENTS.md](AGENTS.md) — Agent configuration guide
- 🎨 [Tab Bar Config](docs/config/lua/config/tab_bar_extra_info.md) — Vertical tab bar customization
- 📦 [Releases](https://github.com/wsyb/wezteam/releases) — Download latest version

---

## About This Project

WezTeam is an enhanced fork of [WezTerm](https://github.com/wezterm/wezterm) (GPU-accelerated cross-platform terminal emulator by [@wez](https://github.com/wez)).

> **Upstream**: [wezterm/wezterm](https://github.com/wezterm/wezterm)
>
> This project follows upstream's [MIT License](LICENSE.md).

### Known Limitations

- Process start/exit does not trigger events; command display updates on title change/mouse/focus change
- OSC 7 CWD may be inaccurate in some scenarios; prefer tab title path

### Code Structure

| Module | Path | Description |
|--------|------|-------------|
| Data Layer | `wezterm-gui/src/termwindow/tab_extra_info.rs` | Path, Git branch, process info |
| UI Layer | `wezterm-gui/src/termwindow/render/fancy_tab_bar.rs` | Info panel rendering |
| Config Layer | `config/src/color.rs` | `ExtraInfoStyle` / `ExtraInfoItemStyle` |
| TeamShell CLI | `tsh/` | Workstation management CLI |
| Protocol Docs | `TeamShellProtocol.md` / `AGENTS.md` | Collaboration protocol |

## License

[MIT License](LICENSE.md)

Original project copyright (c) 2018-Present Wez Furlong
