# 侧边栏 git 分支显示修复方案

## 问题

git 分支信息只有在运行了某些命令之后才会出现在标签栏里，而不是开箱即显。

## 根本原因

1. **CWD 获取依赖 OSC 7**：需要 shell integration 发送 OSC 7 转义序列
2. **首次打开终端时 OSC 7 还未发送**：shell 还未运行第一个 precmd 钩子
3. **Windows 进程检查不可靠**：权限问题导致回退方案失败

## 解决方案

**主动检测路径变化 → 执行 git 命令获取分支名**

不依赖：
- ❌ Shell Integration（OSC 7）
- ❌ 进程检查（不可靠）

而是：
- ✅ 直接执行 `git branch --show-current` 命令
- ✅ 解析输出获取分支名

---

## 实现方案

### 方案 1：同步执行 git 命令（简单，推荐）

**修改文件**：`wezterm-gui/src/termwindow/tab_extra_info.rs`

**修改函数**：`fetch_git_branch_from_path()` （第 143-188 行）

**修改前**：
```rust
fn fetch_git_branch_from_path(path_str: &str) -> Option<String> {
    // ... 路径标准化 ...
    
    let path = Path::new(&path_str);
    let git_dir = find_git_dir(path)?;
    let branch = read_git_branch(&git_dir)?;
    let status = get_git_status(&git_dir)?;
    
    Some(format!("git:{} {}", branch, status))
}
```

**修改后**：
```rust
fn fetch_git_branch_from_path(path_str: &str) -> Option<String> {
    // 1. 路径标准化（处理 Windows 倒序路径）
    let path_str = if path_str.contains(':') && path_str.chars().last() == Some(':') {
        let parts: Vec<&str> = path_str.split('\\').rev().collect();
        parts.join("\\")
    } else {
        path_str.to_string()
    };
    
    log_debug(&format!("fetch_git_branch_from_path: normalized path: {}", path_str));
    
    let path = Path::new(&path_str);
    
    // 2. 检查是否在 git 仓库中
    let git_dir = match find_git_dir(path) {
        Some(dir) => dir,
        None => {
            log_debug(&format!("fetch_git_branch_from_path: no .git in {:?}", path));
            return None;
        }
    };
    
    // 3. 执行 git 命令获取分支名
    let branch = match fetch_git_branch_via_command(path) {
        Some(b) => b,
        None => {
            log_debug(&format!("fetch_git_branch_from_path: git command failed, fallback to .git/HEAD"));
            // 回退到读取 .git/HEAD
            read_git_branch(&git_dir)?
        }
    };
    
    // 4. 获取状态
    let status = match get_git_status(&git_dir) {
        Some(s) => s,
        None => {
            log_debug(&format!("fetch_git_branch_from_path: failed to get status"));
            return None;
        }
    };
    
    let result = format!("git:{} {}", branch, status);
    log_debug(&format!("fetch_git_branch_from_path: {:?}", result));
    Some(result)
}

/// 通过执行 git 命令获取分支名
fn fetch_git_branch_via_command(path: &Path) -> Option<String> {
    use std::process::Command;
    
    // 执行 git branch --show-current
    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(path)
        .output()
        .ok()?;
    
    if !output.status.success() {
        log_debug(&format!("git command failed: {:?}", String::from_utf8_lossy(&output.stderr)));
        return None;
    }
    
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    
    if branch.is_empty() {
        // 可能是 detached HEAD，尝试获取提交哈希
        fetch_git_head_via_command(path)
    } else {
        Some(branch)
    }
}

/// 获取 detached HEAD 的提交信息
fn fetch_git_head_via_command(path: &Path) -> Option<String> {
    use std::process::Command;
    
    // 执行 git rev-parse --short HEAD
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .current_dir(path)
        .output()
        .ok()?;
    
    if !output.status.success() {
        return Some("HEAD".to_string());
    }
    
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Some(format!("HEAD:{}", commit))
}
```

**优点**：
- ✅ 简单直接
- ✅ 不依赖 shell integration
- ✅ 可以获取更详细的 git 信息
- ✅ 有回退方案（读取 .git/HEAD）

**缺点**：
- ⚠️ 需要系统安装 git
- ⚠️ 执行外部命令有一定开销（但可接受）
- ⚠️ 同步执行可能轻微影响性能

---

### 方案 2：异步执行 git 命令（复杂，可选）

**适用场景**：如果同步执行影响性能，可以改为异步

**实现要点**：
1. 使用 `smol::process::Command` 异步执行
2. 使用缓存机制避免频繁执行
3. 异步更新 UI

**代码示例**：
```rust
use smol::process::Command;

async fn fetch_git_branch_via_command_async(path: &Path) -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.args(&["branch", "--show-current"])
        .current_dir(path);
    
    #[cfg(windows)]
    {
        use smol::process::windows::CommandExt;
        cmd.creation_flags(winapi::um::winbase::CREATE_NO_WINDOW);
    }
    
    let output = cmd.output().await.ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

**注意**：异步方案需要重构整个调用链，复杂度较高。

---

## 实现步骤

### 步骤 1：修改 tab_extra_info.rs

在 `fetch_git_branch_from_path()` 函数中添加 git 命令执行逻辑。

### 步骤 2：添加新函数

添加 `fetch_git_branch_via_command()` 和 `fetch_git_head_via_command()` 函数。

### 步骤 3：测试

测试以下场景：
1. ✅ 在 git 仓库中打开终端 → 立即显示分支名
2. ✅ 切换到其他 git 仓库 → 更新分支名
3. ✅ detached HEAD 状态 → 显示提交哈希
4. ✅ 非 git 仓库 → 不显示 git 信息
5. ✅ git 未安装 → 回退到 .git/HEAD

---

## 配置选项（可选）

可以添加配置项控制是否使用 git 命令：

```lua
-- in .wezterm.lua
config.tab_bar_extra_info_use_git_command = true  -- 默认 true
config.tab_bar_extra_info_git_timeout_ms = 1000   -- git 命令超时时间
```

---

## 性能优化

### 优化 1：缓存 git 命令结果

当前已有缓存机制，缓存键为 `(pane_id, cwd)`，缓存时间默认 1000ms。

### 优化 2：限制 git 命令执行频率

可以添加时间间隔限制，避免频繁执行 git 命令。

### 优化 3：超时机制

为 git 命令添加超时，避免长时间等待。

```rust
use std::process::Command;

let output = Command::new("git")
    .args(&["branch", "--show-current"])
    .current_dir(path)
    .timeout(Duration::from_millis(500))  // 500ms 超时
    .output()
    .ok()?;
```

---

## 注意事项

1. **Windows 平台**：
   - 需要确保 git 在 PATH 中
   - 可能需要处理 Windows 特有的路径问题

2. **错误处理**：
   - git 命令失败时优雅降级到 .git/HEAD
   - 记录错误日志便于调试

3. **性能影响**：
   - git 命令执行时间通常 < 10ms
   - 对 UI 渲染影响很小
   - 如有性能问题，可改为异步执行

---

## 总结

**推荐方案**：方案 1（同步执行 git 命令）

**优点**：
- ✅ 不依赖 shell integration
- ✅ 开箱即显
- ✅ 实现简单
- ✅ 性能可接受

**缺点**：
- ⚠️ 需要 git 已安装
- ⚠️ 有回退方案，不会完全失败

这个方案可以解决"git 分支信息不能开箱即显"的问题！
