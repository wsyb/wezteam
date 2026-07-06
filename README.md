# WezTeam - AI Agent Team Collaboration Terminal

[中文文档 →](README.zh-CN.md)

<p align="center">
  <strong>WezTerm + TeamShell</strong>
</p>

<p align="center">
  Run AI agents in terminal tabs. Assign tasks, observe colleagues, communicate, and ship together.
</p>

---

## What Is It?

A terminal emulator built on WezTerm with **heterogeneous AI Agent team collaboration** capabilities.

**You don't need to learn any new commands.** Just chat with your Agent like you normally would.

---

## Why Do I Need This?

### The Problem with Single Agents

You have Claude. It can write code, fix bugs, analyze your project.

But when tasks get complex, a single Agent has inherent limitations:
- **Reviewing its own code** → blind spots
- **One perspective on requirements** → missed edge cases
- **Handing you the result directly** → quality out of your control

### How WezTeam Solves It

Run multiple **different models** in the same terminal and let them collaborate:
- Claude writes the spec → Qwen + Codex review → multiple rounds of iteration
- Claude develops → Codex reviews → Qwen tests → quality gate closed

**One Agent catches 2 issues. A committee of different Agents catches 7.**

You don't need to know how it works internally. You just need to say one sentence.

---

## Who Is This For?

| Audience | Fit | Why |
|----------|-----|-----|
| Developer wanting better code quality | ✅ **Perfect fit** | Multi-Agent review = fewer bugs |
| PM / Architect needing rigorous specs | ✅ **Perfect fit** | Multi-Agent committee = more thorough docs |
| AI-assisted developer | ✅ **Great fit** | Works with a single Agent too |
| Researcher studying AI collaboration | ✅ **Perfect fit** | Transparent process = observable experiment |
| Complete beginner with AI tools | ⚠️ **Try it** | Start with a single Agent first |

**You can use WezTeam even with just one Claude.**

---

## How It Works

Think of it like running a company:

- **WezTeam** = Your office (the terminal)
- **Agent** = Your employees (Claude, Codex, Gemini, Qwen, etc.)
- **TeamShell** = Company rules (the collaboration protocol)
- **tsh** = Internal management tools (Agents handle this themselves)

**Core concept: Heterogeneous Agent collaboration**

Run multiple Agents with **different models** in the same terminal. Let them do what a **single Agent cannot**.

Each Agent runs in an **independent tab**. You see the **entire process in real time**.

---

### Scenario 1: Multi-Agent Committee Review

**Single Agent**: writes and reviews its own work → finds 2 issues

**WezTeam**: multiple Agents review each other's work → finds 7 issues

```
You: "Help me design a user authentication system"

  ↓ Claude in Tab 1 produces the first draft

[Tab 2 - Qwen] [Tab 3 - Codex]
      ↓              ↓
   Start review   Start review

[Tab 2 - Qwen]
"Spec v1, please review"
→ "Passwords use MD5 hashing, should use bcrypt + salt, Score: 6/10"

[Tab 3 - Codex]
"Spec v1, please review"
→ "Login endpoint missing rate limiting, should add Redis brute-force protection, Score: 5/10"

[Tab 1 - Claude]
→ "Noted, I'll update to v2"

[Tab 2 - Qwen]
→ "v2 fixes password security, but concurrency has race conditions, should add distributed lock, Score: 7/10"

[Tab 3 - Codex]
→ "Architecture is solid, but error code conventions are inconsistent, Score: 7/10"

  ↓ [After several rounds]

[Tab 1 - Claude]
→ "Spec v3 is complete"
→ Reports to you: Overall score 8.5/10, 7 issues fixed total
```

**Key value**:
- **Different models, different perspectives**: Claude's architectural thinking + Qwen's engineering rigor + Codex's practical experience
- **Mutual challenge, mutual review**: Not one-sided output, but multi-round discussion
- **Transparent process**: You **see the entire review** in real time, tab by tab

---

### Scenario 2: Develop → Review → Test Pipeline

**Single Agent**: writes code → hands it to you → you find issues manually

**WezTeam**: write → review → fix → test → fix → pass → deliver

```
You: "Help me implement the login feature"

  ↓ Claude in Tab 1 handles development

Claude → finishes code → writes to file

  ↓ Codex in Tab 2 handles review

Codex → reads code → finds 3 issues:
  - Missing input validation
  - Incomplete error handling
  - Performance can be optimized

Codex → [Tab 1] tells Claude: "Please fix the above issues"
Claude → fixes → Codex → second review → passed

  ↓ Qwen in Tab 3 handles testing

Qwen → runs tests → finds edge case failures
Qwen → [Tab 1] tells Claude: "Tests failing, please fix"
Claude → fixes → Qwen → re-runs tests → passed

  ↓ Claude reports to you

Claude: "Login feature complete, passed Codex review and Qwen testing, ready to merge"
```

**Key value**:
- **Specialized roles**: Development, review, and testing each handled by the right Agent
- **Quality gate**: Review + test double-check, not just "done when written"
- **Single direction flow**: Each Agent stays in its lane
- **Transparent process**: Three tabs, real-time view of the full development and validation cycle

---

### What Both Scenarios Have in Common

| Characteristic | Description |
|---------------|-------------|
| **Heterogeneous Agents** | Claude + Qwen + Codex — different models, different strengths |
| **Mutual review** | Agents don't just execute — they challenge, audit, and score |
| **Multi-round iteration** | Not one pass, but converging toward rigor |
| **Quality first** | Rigorous, reliable, deliverable — not fast |
| **Transparent process** | Every tab visible, fully traceable |
| **Zero burden on you** | You assign the task, the process runs itself |

**You don't need to know `tsh` commands. You don't manage Agents. You don't manually coordinate.**

You just say: **"Help me build a login feature"**

The rest is up to the Agents.

---

## Quick Start

### Download & Install

**Windows**: Download `WezTerm-*-setup.exe` → Double-click to install

**macOS**: Download `WezTerm-*-macos.zip` → Extract and drag to `/Applications`

**Linux**:
- Ubuntu/Debian: `wget ... && tar -xf ... && sudo dpkg -i .`
- Fedora/CentOS: `wget ... && sudo dnf install ...`
- Universal: Download AppImage, `chmod +x`, then run

---

### Configure (One-Time Setup)

```bash
tsh init
```

This automatically writes the collaboration protocol into your Agent's configuration files.

**Having trouble?** Open WezTeam, launch your Agent in a new tab, and simply say:

```
Please read @TeamShellProtocol.md
```

The Agent will load the protocol — same effect as `tsh init`.

---

### Start Using

**Launch a single Agent**:

```bash
claude    # Claude
# codex   # Codex
# gemini  # Gemini
# qwen    # Qwen
```

**Launch multiple different Agents** for team collaboration:

```bash
# Tab 1: Claude handles architecture design
claude

# Tab 2: Qwen handles frontend development
qwen

# Tab 3: Codex handles backend development
codex
```

When an Agent reads the protocol for the first time, it will **automatically complete onboarding** and report to you.

Then just chat like you would with any Agent:

```
"Help me build a login feature"
```

The Agent will:
- Understand your needs
- Use the TeamShell protocol to communicate and coordinate with other Agents
- Report results back to you

**You don't need to know any internal details.**

---

## For Developers

If you want to **build from source** or **contribute**, see:

📖 **[BUILD.md](BUILD.md)** (English, standalone)

---

## Learn More

- 📖 **[TeamShellProtocol.md](TeamShellProtocol.md)** — Full collaboration protocol (for Agents)
- 📖 **[AGENTS.md](AGENTS.md)** — Agent configuration guide
- 🎨 **[Tab Bar Config](docs/config/lua/config/tab_bar_extra_info.md)** — Customize the vertical tab bar
- 📦 **[Releases](https://github.com/wsyb/wezteam/releases)** — Download the latest version

---

## About This Project

WezTeam is an enhanced fork of [WezTerm](https://github.com/wezterm/wezterm) by [@wez](https://github.com/wez).

- **Upstream**: https://github.com/wezterm/wezterm
- **License**: [MIT](LICENSE.md)
