@echo off
setlocal

cd /d "%~dp0"

rem set CARGO_BUILD_JOBS=8

cargo clean

echo Building release binaries...
cargo build -p wezterm --release
if errorlevel 1 goto failed
cargo build -p wezterm-gui --release
if errorlevel 1 goto failed
cargo build -p wezterm-mux-server --release
if errorlevel 1 goto failed
cargo build -p strip-ansi-escapes --release
if errorlevel 1 goto failed
cargo build -p teamshell-cli --release
if errorlevel 1 goto failed

echo.
echo Build complete!
exit /b 0

:failed
echo.
echo Build failed.
pause
exit /b 1
