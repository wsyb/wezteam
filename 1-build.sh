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

# Detect X11 development libraries and select features accordingly
if [ "$PLATFORM" = "Linux" ]; then
  if pkg-config --exists x11-xcb 2>/dev/null; then
    GUI_FEATURES="vendored-fonts,wayland,x11"
    echo "  X11 development libraries found: building with X11 + Wayland support"
  else
    GUI_FEATURES="vendored-fonts,wayland"
    echo "  X11 development libraries NOT found: building with Wayland-only support"
    echo "  (Install libx11-xcb-dev or run ./get-deps to enable X11 support)"
  fi
fi

cargo build -p wezterm --release
if [ -n "$GUI_FEATURES" ]; then
  cargo build -p wezterm-gui --release --no-default-features --features "$GUI_FEATURES"
else
  cargo build -p wezterm-gui --release
fi
cargo build -p wezterm-mux-server --release
cargo build -p strip-ansi-escapes --release
cargo build -p teamshell-cli --release

echo
echo "✅ Build complete! Binaries in target/release/"
