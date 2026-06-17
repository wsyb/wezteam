#!/bin/bash
set -e

cd "$(dirname "$0")"

TAG_NAME=${TAG_NAME:-dev-test}
PKG_DIR="WezTerm-linux-$TAG_NAME"
TAR_NAME="WezTerm-linux-$TAG_NAME.tar.gz"
TARGET_DIR=${TARGET_DIR:-target}

echo "📦 Packaging Linux tarball..."

rm -rf "$PKG_DIR" "$TAR_NAME"
mkdir -p "$PKG_DIR"

# Copy binaries
for bin in wezterm wezterm-mux-server wezterm-gui strip-ansi-escapes tsh; do
    if [[ -f "$TARGET_DIR/release/$bin" ]]; then
        cp "$TARGET_DIR/release/$bin" "$PKG_DIR/"
    else
        echo "⚠️  Warning: $bin not found in $TARGET_DIR/release/"
    fi
done

# Copy extra files
cp assets/open-wezterm-here "$PKG_DIR/"
cp assets/wezterm.desktop "$PKG_DIR/"
cp assets/wezterm.appdata.xml "$PKG_DIR/"
cp assets/icon/terminal.png "$PKG_DIR/wezterm-icon.png"

# Shell completions & integration
cp -r assets/shell-completion "$PKG_DIR/"
cp -r assets/shell-integration "$PKG_DIR/"

# Terminfo
tic -xe wezterm -o "$PKG_DIR/terminfo" termwiz/data/wezterm.terminfo 2>/dev/null || true

# Create tarball
tar czf "$TAR_NAME" "$PKG_DIR/"

echo
echo "✅ Package complete: $TAR_NAME"
