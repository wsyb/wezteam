#!/bin/bash
set -e

cd "$(dirname "$0")"

# Auto-detect platform
case "$OSTYPE" in
  darwin*)
    PKG_SCRIPT="./2-pkg-macos.sh"
    ;;
  linux-gnu*|linux*)
    PKG_SCRIPT="./2-pkg-linux.sh"
    ;;
  msys|cygwin|win32)
    echo "⚠️  For Windows packaging, please run 2-pkg.cmd directly"
    exit 1
    ;;
  *)
    echo "❓ Unknown platform: $OSTYPE"
    exit 1
    ;;
esac

echo "🚀 Full release build..."

./1-build.sh
"$PKG_SCRIPT"

echo
echo "✅ Release complete!"
