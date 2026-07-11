# WezTeam - AI Agent Team Collaboration Terminal

[中文文档 →](README.zh-CN.md)

---

## What Is It?

A terminal emulator built on WezTerm with **heterogeneous AI Agent team collaboration** capabilities.

**You don't need to learn any new commands.** Just chat with your Agent like you normally would.

---

## Why Do You Need It?

### The Limits of a Single Agent

You have Claude. It can write code, fix bugs, and analyze your project.

But as tasks grow more complex, a single Agent has inherent limits:
- **Reviewing its own code** → easy to miss blind spots
- **Analyzing requirements from one perspective** → boundary cases slip through
- **Handing you the finished product** → quality is out of your control

### How WezTeam Solves This

Let **different models** collaborate in the same terminal:
- Claude writes the spec → Qwen + Codex review → iterate together
- Claude develops → Codex reviews → Qwen tests → quality gate

**A single Agent finds 2 issues. A multi-Agent committee finds 7.**

You don't need to know how the internals work. Just say one sentence.

---

## Who Is This For?

| Who | Fit | Why |
|-----|-----|-----|
| Developer wanting better code quality | ✅ **Excellent** | Multi-Agent review = fewer bugs |
| PM / Architect needing rigorous specs | ✅ **Excellent** | Multi-Agent committee = more thorough docs |
| AI-assisted programmer | ✅ **Good** | Works great even with a single Agent |
| AI collaboration researcher | ✅ **Excellent** | Transparent process = observable experiment |
| Complete AI tool beginner | ⚠️ **Try it** | Start with a single Agent first |

**Even if you only have Claude, WezTeam works for you.**

---

## How It Works

Think of it like running a company:

- **WezTeam** = your office (the terminal)
- **Agent** = your employees (Claude, Codex, Gemini, Qwen, etc.)
- **TeamShell** = company rules (the collaboration protocol)
- **tsh** = internal management tool (Agents use this themselves)

**Core idea: heterogeneous Agent collaboration**

Run multiple Agents with **different models** in the same terminal — doing things a single Agent cannot.

Each Agent runs in its own tab. **You see everything in real time.**

---

### Scenario 1: Multi-Agent Committee Review

**Single Agent** → writes and reviews its own work → finds 2 issues

**WezTeam** → multiple Agents review each other's work → finds 7 issues

```
You: "Help me design a user login system"

  ↓ Claude produces v1 of the requirements doc (tab 1)

[Tab 2 - Qwen]              [Tab 3 - Codex]
       ↓                          ↓
    Start review               Start review

[Tab 2 - Qwen]
"Requirements doc v1, please review"
→ "Passwords use MD5 hashing, should use bcrypt + salt. Score: 6/10"

[Tab 3 - Codex]
"Requirements doc v1, please review"
→ "Login endpoint lacks rate limiting, needs Redis brute-force protection. Score: 5/10"

[Tab 1 - Claude]
→ "Got it, I'll update to v2"

[Tab 2 - Qwen]
→ "v2 fixes password security, but concurrent access has race conditions. Needs distributed lock. Score: 7/10"

[Tab 3 - Codex]
→ "Architecture is solid, but error code conventions are inconsistent. Score: 7/10"

  ↓ [After several rounds of discussion]

[Tab 1 - Claude]
→ "Requirements doc v3 is complete"
→ Reports to you: Overall score 8.5/10, 7 issues resolved total
```

**Key value**:
- **Different models, different perspectives**: Claude's architecture thinking + Qwen's engineering rigor + Codex's practical experience
- **Mutual critique and review**: Not one-sided output, but iterative discussion
- **Fully transparent**: You see the entire review process **in real time** on your terminal tabs

---

### Scenario 2: Develop → Review → Test Pipeline

**Single Agent** → hands you finished code → you manually find issues

**WezTeam** → write → review → fix → test → fix → pass → deliver

```
You: "Help me implement a login feature"

  ↓ Claude develops (tab 1)

Claude → finishes code → writes to file

  ↓ Codex reviews (tab 2)

Codex → reads code → finds 3 issues:
  - missing input validation
  - incomplete error handling
  - performance can be optimized

Codex → [tab 1] tells Claude: "Please fix the above"
Claude → fixes → Codex → second review → passed

  ↓ Qwen tests (tab 3)

Qwen → runs tests → finds edge case failure
Qwen → [tab 1] tells Claude: "Tests failed, please fix"
Claude → fixes → Qwen → re-runs → passed

  ↓ Claude reports to you

Claude: "Login feature complete, Codex review passed, Qwen tests passed, ready to merge"
```

**Key value**:
- **Specialized roles**: development, review, and testing each have their lane
- **Quality closed-loop**: review + test double-gate, not just "done when written"
- **One-way flow**: each Agent focuses on their role, no cross-interference
- **Fully transparent**: three tabs, real-time view of the entire development and validation process

---

### What Both Scenarios Have in Common

| Trait | Description |
|-------|-------------|
| **Heterogeneous Agents** | Claude + Qwen + Codex — different models, different strengths |
| **Mutual review** | Agents don't just execute — they challenge, audit, and score |
| **Iterative refinement** | Not a one-shot pass, but converging toward rigor |
| **Quality-first** | Prioritize thoroughness, reliability, and deliverability |
| **Transparent process** | every tab visible in real time, fully traceable |
| **Zero user burden** | You just assign the task; the process runs itself |

**You don't need to know `tsh` commands, don't need to manage Agents, don't need to manually coordinate.**

Just say: **"Help me build a login feature"**

The Agents handle the rest.

---

## Quick Start

### Download

Download the latest release from the [Releases page](https://github.com/wsyb/wezteam/releases).

**Windows**: Download `WezTerm-*-setup.exe` → double-click to install

**macOS**: Download `WezTerm-macos-*.zip` → extract and drag to `/Applications`

**Linux**:
```bash
# Ubuntu/Debian — download .deb and install
wget https://github.com/wsyb/wezteam/releases/download/v1.0.0/wezterm-1.0.0.Ubuntu24.04.deb
sudo dpkg -i wezterm-1.0.0.Ubuntu24.04.deb

# Fedora/CentOS — download .rpm and install
wget https://github.com/wsyb/wezteam/releases/download/v1.0.0/wezterm-1.0.0-1.fedora41.x86_64.rpm
sudo dnf install ./wezterm-1.0.0-1.fedora41.x86_64.rpm

# Universal — download AppImage
chmod +x WezTerm-*.AppImage
./WezTerm-*.AppImage
```

---

### Configure (One-Time)

```bash
tsh init
```

This automatically writes the collaboration protocol to your Agent configuration files.

**If you run into issues?** Open WezTeam, start an Agent in a new tab, and simply say in the conversation:

```
Please read @TeamShellProtocol.md
```

The Agent will read the protocol directly. Same effect as `tsh init`.

---

### Start Using

**Launch a single Agent**:

```bash
claude    # Claude
# codex   # Codex
# gemini  # Gemini
# qwen    # Qwen
```

**Launch multiple different Agents** to collaborate:

```bash
# Tab 1: Claude handles architecture
claude

# Tab 2: Qwen handles frontend
qwen

# Tab 3: Codex handles backend
codex
```

When an Agent sees the protocol for the first time, it will **automatically complete onboarding** and report to you.

After that, just chat like you normally would:

```
"Help me build a login feature"
```

The Agent will:
- Understand your request
- Use the TeamShell protocol to communicate and coordinate with other Agents
- Report the results back to you

**You don't need to know any internal details.**

---

## Developer Documentation

If you want to **build from source** or **contribute**, see:

📖 **[BUILD.md](BUILD.md)** (English, standalone document)

---

## More Resources

- 📖 **[TeamShellProtocol.md](TeamShellProtocol.md)** — Full collaboration protocol (for Agents)
- 📖 **[AGENTS.md](AGENTS.md)** — Agent configuration guide
- 🎨 **[Tab Bar Config](docs/config/lua/config/tab_bar_extra_info.md)** — Customize the vertical tab bar
- 📦 **[Releases](https://github.com/wsyb/wezteam/releases)** — Download the latest version

---

## About This Project

An enhanced fork of [WezTerm](https://github.com/wezterm/wezterm).

- **Upstream**: [wezterm/wezterm](https://github.com/wezterm/wezterm)
- **License**: [MIT](LICENSE.md)
