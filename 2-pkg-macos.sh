#!/bin/bash
set -e

cd "$(dirname "$0")"

TAG_NAME=${TAG_NAME:-dev-test}
ZIP_DIR="WezTerm-macos-$TAG_NAME"
ZIP_NAME="WezTerm-macos-$TAG_NAME.zip"
TARGET_DIR=${TARGET_DIR:-target}

echo "📦 Packaging macOS app bundle..."

rm -rf "$ZIP_DIR" "$ZIP_NAME"
mkdir -p "$ZIP_DIR"

# Copy template app bundle
cp -r assets/macos/WezTerm.app "$ZIP_DIR/"

# Omit MetalANGLE for now (laggy compared to CGL)
rm -f "$ZIP_DIR/WezTerm.app/"*.dylib

# Create directories
mkdir -p "$ZIP_DIR/WezTerm.app/Contents/MacOS"
mkdir -p "$ZIP_DIR/WezTerm.app/Contents/Resources"

# Copy resources
cp -r assets/shell-integration/* "$ZIP_DIR/WezTerm.app/Contents/Resources/"
cp -r assets/shell-completion "$ZIP_DIR/WezTerm.app/Contents/Resources/"

# Build & copy terminfo
tic -xe wezterm -o "$ZIP_DIR/WezTerm.app/Contents/Resources/terminfo" termwiz/data/wezterm.terminfo

# Copy binaries
for bin in wezterm wezterm-mux-server wezterm-gui strip-ansi-escapes tsh; do
    if [[ -f "$TARGET_DIR/release/$bin" ]]; then
        cp "$TARGET_DIR/release/$bin" "$ZIP_DIR/WezTerm.app/Contents/MacOS/$bin"
    else
        echo "⚠️  Warning: $bin not found in $TARGET_DIR/release/"
    fi
done

# Zip it up
zip -r "$ZIP_NAME" "$ZIP_DIR"

echo
echo "✅ Package complete: $ZIP_NAME"
