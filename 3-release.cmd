@echo off
setlocal

cd /d "%~dp0"

call 1-build.cmd
if errorlevel 1 exit /b 1

call 2-pkg-windows.cmd
if errorlevel 1 exit /b 1

echo.
echo Release complete!
