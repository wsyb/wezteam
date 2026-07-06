# Building WezTeam from Source

This guide is for **developers who want to contribute to WezTeam** or need to build from source for any reason.

If you just want to **use WezTeam**, please download the pre-built installer from [Releases](https://github.com/wsyb/wezteam/releases/latest) instead.

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- **Windows**: Visual Studio Build Tools
- **macOS**: Xcode Command Line Tools
- **Linux**: See [upstream build docs](https://github.com/wezterm/wezterm/blob/main/README.md#building-from-source)

### Verify Your Setup

```bash
rustc --version    # Should show stable version
cargo --version    # Should show cargo version
```

---

## Clone the Repository

```bash
git clone https://github.com/wsyb/wezteam.git
cd wezteam
```

If you plan to contribute, consider forking the repository first and cloning your fork:

```bash
git clone https://github.com/<your-username>/wezteam.git
cd wezteam
git remote add upstream https://github.com/wsyb/wezteam.git
```

---

## Build

```bash
cargo build --release --package wezterm-gui
```

**First build may take 5-10 minutes** as it compiles all dependencies. Subsequent builds are much faster.

### Build Specific Components

```bash
# Build wezterm-gui only
cargo build --release --package wezterm-gui

# Build with all features
cargo build --release --package wezterm-gui --all-features

# Build specific target
cargo build --release --target x86_64-unknown-linux-gnu
```

### Build All Packages

```bash
cargo build --release --all
```

This builds all packages in the workspace including:
- `wezterm` — Terminal emulator
- `wezterm-gui` — GUI frontend
- `wezterm-mux-server` — Multiplexer server (headless)
- `strip-ansi-escapes` — ANSI escape sequence stripper
- `teamshell-cli` — TeamShell CLI (tsh)

---

## Run

```bash
# Run wezterm-gui
cargo run --release --package wezterm-gui

# Or run the binary directly after building
./target/release/wezterm-gui      # macOS/Linux
./target/release/wezterm-gui.exe  # Windows
```

---

## Build Installer Packages

### Prerequisites

- **Windows**: [Inno Setup 6](https://jrsoftware.org/isdl.php)
- **macOS**: Xcode Command Line Tools + Apple Developer certificate (for signing)
- **Linux**: Standard build tools + packaging utilities

### Windows

```cmd
:: One-click build + package
3-release.cmd

:: Or step-by-step
1-build.cmd && 2-pkg-windows.cmd
```

### Linux

```bash
cargo build -p wezterm --release -p wezterm-gui --release -p wezterm-mux-server --release -p strip-ansi-escapes --release
TAG_NAME=v0.1.0 bash 2-pkg-linux.sh
```

### macOS

```bash
cargo build -p wezterm --release -p wezterm-gui --release -p wezterm-mux-server --release -p strip-ansi-escapes --release
TAG_NAME=v0.1.0 bash 2-pkg-macos.sh
```

---

## Install tsh CLI (TeamShell)

The `tsh` CLI is included in the WezTeam repository. To build and install it:

```bash
# Install from source
cargo install --path tsh

# Or use it directly without installing
cargo run --release --package teamshell-cli -- open "My Assistant" -- claude
```

---

## Testing

Run the test suite:

```bash
cargo test                      # All tests
cargo test --package wezterm-gui # Specific package
cargo nextest run --all         # Using nextest (faster)
```

---

## Troubleshooting

### Build fails with OpenSSL errors

Try using the vendored OpenSSL feature:

```bash
cargo build --release --package wezterm-gui --features=openssl/vendored
```

### macOS: Code signing required

To run WezTeam on macOS without signing:

```bash
codesign --remove-signature target/release/wezterm-gui.app
xattr -cr target/release/wezterm-gui.app
```

**Note**: This is only for development. For distribution, code signing is required.

### Windows: MSVC linker errors

Ensure you have Visual Studio Build Tools installed with the "Desktop development with C++" workload.

### Slow builds

Use `sccache` to speed up rebuilds:

```bash
# Install sccache
cargo install sccache

# Enable sccache
export RUSTC_WRAPPER=sccache
cargo build --release
```

---

## Understanding the Codebase

### Key Directories

| Directory | Description |
|-----------|-------------|
| `wezterm-gui/` | GUI frontend (main application) |
| `wezterm/` | Core terminal emulator logic |
| `wezterm-mux-server/` | Headless multiplexer server |
| `wezterm-mux-server-impl/` | Multiplexer server implementation |
| `config/` | Configuration parsing and handling |
| `termwiz/` | Terminal emulation library |
| `term/` | Terminal state management |
| `tsh/` | TeamShell CLI tool |
| `assets/` | Application assets (icons, shell integration, etc.) |

### Documentation

- [TeamShell Protocol](TeamShellProtocol.md) — Collaboration protocol specification
- [AGENTS.md](AGENTS.md) — Agent configuration guide
- [Vertical Tab Bar Config](docs/config/lua/config/tab_bar_extra_info.md) — Tab bar customization

---

## Contributing

We welcome contributions! Please follow these steps:

1. **Fork the repository** and create a feature branch
2. **Make your changes** with clear commit messages
3. **Test your changes** (`cargo test`, `cargo nextest run --all`)
4. **Submit a Pull Request** to the `main` branch

### Code Style

- Follow Rust conventions (`cargo fmt`, `cargo clippy`)
- Write clear commit messages
- Add tests for new functionality
- Update documentation as needed

---

## Upstream WezTerm

WezTeam is a fork of [WezTerm](https://github.com/wezterm/wezterm) by [@wez](https://github.com/wez).

- **Upstream**: https://github.com/wezterm/wezterm
- **License**: MIT (same as upstream)
- **Copyright**: Original project (c) 2018-Present Wez Furlong

When contributing, please be aware that we aim to keep WezTeam aligned with upstream while adding TeamShell-specific features.
