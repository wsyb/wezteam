# wezteam Windows Installer Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Change the Windows Inno Setup package branding to `wezteam` while preserving the original WezTerm binaries and adding a `wezteam.exe` launcher copy for shortcuts.

**Architecture:** This is a packaging-only change in `ci/windows-installer.iss`. The installer display name, output filename, default install directory, shortcuts, and post-install launch target will use `wezteam`; the original `wezterm.exe`, `wezterm-gui.exe`, and related executables remain installed unchanged to reduce runtime risk.

**Tech Stack:** Inno Setup Script (`.iss`), Windows release artifacts under `target\release`, PowerShell verification, optional `ISCC.exe` packaging.

---

## File Structure

- Modify: `ci/windows-installer.iss`
  - Owns Windows installer metadata, packaged files, shortcuts, registry context-menu entries, and post-install launch behavior.
  - Required changes:
    - `MyAppName` becomes `wezteam`.
    - `MyAppExeName` becomes `wezteam.exe`.
    - `OutputBaseFilename` becomes `wezteam-Setup`.
    - Add one extra `[Files]` entry that copies `..\target\release\wezterm-gui.exe` to `{app}\wezteam.exe` via `DestName`.
    - Keep existing `wezterm-gui.exe` entry so existing CLI/runtime expectations remain compatible.

## Success Criteria

- `ci/windows-installer.iss` still packages all original files.
- Installer UI and install directory use `wezteam` through `MyAppName`.
- Generated installer filename is `wezteam-Setup.exe`.
- Start menu, desktop shortcut, uninstall icon, and post-install launch target point to `{app}\wezteam.exe`.
- No source-level Rust binary rename is attempted.

### Task 1: Update Inno Setup branding and launcher target

**Files:**
- Modify: `ci/windows-installer.iss:5-9`
- Modify: `ci/windows-installer.iss:28-31`
- Modify: `ci/windows-installer.iss:45-56`

- [ ] **Step 1: Inspect current installer script before editing**

Run:

```powershell
Get-Content ci\windows-installer.iss -TotalCount 70
```

Expected: the file contains these current definitions:

```iss
#define MyAppName "WezTerm"
#define MyAppExeName "wezterm-gui.exe"
OutputBaseFilename=WezTerm-Setup
Source: "..\target\release\wezterm-gui.exe"; DestDir: "{app}"; Flags: ignoreversion
```

- [ ] **Step 2: Change the installer display name and shortcut executable**

Replace this block:

```iss
#define MyAppName "WezTerm"
;#define MyAppVersion "1.5"
#define MyAppPublisher "Wez Furlong"
#define MyAppURL "http://wezterm.org"
#define MyAppExeName "wezterm-gui.exe"
```

with:

```iss
#define MyAppName "wezteam"
;#define MyAppVersion "1.5"
#define MyAppPublisher "Wez Furlong"
#define MyAppURL "http://wezterm.org"
#define MyAppExeName "wezteam.exe"
```

- [ ] **Step 3: Change generated installer filename**

Replace:

```iss
OutputBaseFilename=WezTerm-Setup
```

with:

```iss
OutputBaseFilename=wezteam-Setup
```

- [ ] **Step 4: Add a `wezteam.exe` copy while preserving `wezterm-gui.exe`**

Replace this `[Files]` excerpt:

```iss
Source: "..\target\release\wezterm.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\wezterm-gui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\wezterm-mux-server.exe"; DestDir: "{app}"; Flags: ignoreversion
```

with:

```iss
Source: "..\target\release\wezterm.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\wezterm-gui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\wezterm-gui.exe"; DestDir: "{app}"; DestName: "wezteam.exe"; Flags: ignoreversion
Source: "..\target\release\wezterm-mux-server.exe"; DestDir: "{app}"; Flags: ignoreversion
```

- [ ] **Step 5: Verify the exact changed lines**

Run:

```powershell
Select-String -Path ci\windows-installer.iss -Pattern 'MyAppName|MyAppExeName|OutputBaseFilename|DestName: "wezteam.exe"|\{#MyAppExeName\}' -Context 0,0
```

Expected output includes:

```text
#define MyAppName "wezteam"
#define MyAppExeName "wezteam.exe"
OutputBaseFilename=wezteam-Setup
Source: "..\target\release\wezterm-gui.exe"; DestDir: "{app}"; DestName: "wezteam.exe"; Flags: ignoreversion
UninstallDisplayIcon={app}\{#MyAppExeName}
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; AppUserModelID: "org.wezfurlong.wezterm"
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
```

- [ ] **Step 6: Review diff scope**

Run:

```powershell
git diff -- ci\windows-installer.iss
```

Expected: the diff only changes the app name, executable macro, output filename, and adds the `DestName: "wezteam.exe"` file entry.

- [ ] **Step 7: Optional package syntax check with Inno Setup**

Run this only if `ISCC.exe` is installed and the required `target\release` files exist:

```powershell
ISCC.exe ci\windows-installer.iss /DMyAppVersion=20260615
```

Expected: Inno Setup completes successfully and writes `wezteam-Setup.exe` in the repository parent output location configured by `OutputDir=..`.

If `ISCC.exe` is not installed or release artifacts are missing, record that packaging verification was skipped for that reason.

- [ ] **Step 8: Commit this packaging-only change**

Run:

```powershell
git add ci\windows-installer.iss docs\superpowers\plans\2026-06-15-wezteam-windows-installer-rename.md
git commit -m "build: brand windows installer as wezteam"
```

Expected: a new commit containing only `ci/windows-installer.iss` and this plan file.

## Self-Review

- Spec coverage:方案 A 覆盖安装包显示名、安装目录、快捷方式显示名、输出安装包文件名；方案 B 覆盖通过 `DestName` 额外生成 `wezteam.exe` 并让快捷方式启动它。
- Placeholder scan: no `TBD`, `TODO`, or vague implementation steps remain.
- Type/name consistency: all Inno macro names are existing macros; `MyAppExeName` consistently points to `wezteam.exe`; original `wezterm-gui.exe` is still installed.
