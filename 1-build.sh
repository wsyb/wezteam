#!/bin/bash
set -e

cd "$(dirname "$0")"

# Auto-detect platform
case "$OSTYPE" in
  darwin*)
    PLATFORM="macOS"
    ;;
  linux-gnu*|linux*)
    PLATFORM="Linux"
    ;;
  msys|cygwin|win32)
    PLATFORM="Windows"
    ;;
  *)
    echo "❓ Unknown platform: $OSTYPE"
    exit 1
    ;;
esac

echo "🔨 Building release binaries for $PLATFORM..."

cargo build -p wezterm --release
cargo build -p wezterm-gui --release
cargo build -p wezterm-mux-server --release
cargo build -p strip-ansi-escapes --release
cargo build -p teamshell-cli --release

echo
echo "✅ Build complete! Binaries in target/release/"
