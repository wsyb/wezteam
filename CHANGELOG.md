# Changelog

## v1.0.0 (2026-07-11)

### TeamShell — Heterogeneous Agent Collaboration

- **TeamShell protocol** — full collaboration protocol enabling heterogeneous AI Agent teams (Claude + Qwen + Codex + Gemini) to work together in the same terminal
- **`tsh` CLI tool** — team management: `status`, `view`, `type`, `report`, `query`, `open`, `close`, `name`, `init`
- **TCP loopback IPC** — migrated from `interprocess` to fixed-port TCP loopback (127.0.0.1:31415) for stable cross-process communication
- **Shared IPC layer** — extracted `teamshell-ipc` crate for reusable IPC contracts between Agents and the mux server

### Protocol & Agent Experience

- **Onboarding verification** — Agents self-validate identity via `TEAMSH_TAB_ID`, `TEAMSH_NAME`, `TEAMSH_PLATFORM` env vars
- **Message routing** — `[TeamShell Message]` prefix system for Agent-to-Agent communication
- **Status reporting** — shared dashboard with `tsh status` / `tsh query` for real-time team awareness

### Platform & Build

- **X11 optional dependency** — `x11` feature flag for Wayland-focused builds
- **Wayland CSD/SSD** — client-side and server-side decoration adaptive support
- **Wayland cursor fix** — pointer visibility issue resolved
- **Cross-platform CI** — GitHub Actions for Ubuntu 22.04/24.04/26.04, Debian 12, Fedora 41, CentOS 9, macOS, Windows

### Infrastructure

- **CI migrated** — repository condition and tag patterns updated for `wsyb/wezteam` fork
- **Multi-platform packaging** — deb, rpm, tar.xz, AppImage, macOS zip, Windows exe/zip

### About

This is the initial release of WezTeam, a fork of [WezTerm](https://github.com/wezterm/wezterm) adding heterogeneous AI Agent team collaboration capabilities.
