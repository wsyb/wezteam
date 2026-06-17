@echo off
setlocal

cd /d "%~dp0"

set "ISCC="
where iscc.exe >nul 2>nul
if not errorlevel 1 set "ISCC=iscc.exe"

if not defined ISCC if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" set "ISCC=%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe"
if not defined ISCC if exist "%ProgramFiles%\Inno Setup 6\ISCC.exe" set "ISCC=%ProgramFiles%\Inno Setup 6\ISCC.exe"

if not defined ISCC (
  echo Could not find Inno Setup compiler ISCC.exe.
  echo Please install Inno Setup 6 or add ISCC.exe to PATH.
  pause
  exit /b 1
)

echo Packaging wezteam Windows installer...
echo Using: %ISCC%
"%ISCC%" -DMyAppVersion=dev-test -Fwezteam-Setup ci\windows-installer.iss
if errorlevel 1 (
  echo.
  echo Packaging failed.
  pause
  exit /b 1
)

echo.
echo Packaging complete: wezteam-Setup.exe
pause
