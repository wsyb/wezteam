@echo off
setlocal

cd /d "%~dp0"

echo Building release binaries...
cargo build -p wezterm --release
if errorlevel 1 goto failed
cargo build -p wezterm-gui --release
if errorlevel 1 goto failed
cargo build -p wezterm-mux-server --release
if errorlevel 1 goto failed
cargo build -p strip-ansi-escapes --release
if errorlevel 1 goto failed
cargo build -p tsh --release
if errorlevel 1 goto failed

call "%~dp0package-windows-installer.cmd"
exit /b %errorlevel%

:failed
echo.
echo Build failed.
pause
exit /b 1
