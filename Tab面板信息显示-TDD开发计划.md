# Tab 面板信息显示 - TDD 开发计划

## 🎯 目标

使用测试驱动开发（TDD）方式，100% 符合需求文档中的所有要求。

---

## 📋 TDD 开发流程

对于每个功能点：
1. **Red**：先写测试（测试失败）
2. **Green**：写最少代码让测试通过
3. **Refactor**：重构优化代码
4. **Repeat**：重复下一个功能点

---

## 🧪 测试用例设计

### 模块 1：标题路径判断（`is_likely_path`）

#### 测试用例 1.1：绝对路径识别
```rust
#[test]
fn test_absolute_path_unix() {
    assert!(is_likely_path("/home/user/project"));
    assert!(is_likely_path("/usr/local/bin"));
}

#[test]
fn test_absolute_path_windows() {
    assert!(is_likely_path("C:\\Users\\test"));
    assert!(is_likely_path("D:\\work\\project"));
    assert!(is_likely_path("C:/Users/test"));
}
```

#### 测试用例 1.2：相对路径识别
```rust
#[test]
fn test_relative_path() {
    assert!(is_likely_path("src/main.rs"));
    assert!(is_likely_path("lib/mod.rs"));
    assert!(is_likely_path("..\\parent\\file.txt"));
}
```

#### 测试用例 1.3：非路径识别
```rust
#[test]
fn test_non_path() {
    assert!(!is_likely_path("claude"));
    assert!(!is_likely_path("vim"));
    assert!(!is_likely_path("bash"));
    assert!(!is_likely_path(""));
}
```

#### 测试用例 1.4：边界情况
```rust
#[test]
fn test_edge_cases() {
    // git:main 包含冒号，但不是路径
    assert!(!is_likely_path("git:main"));
    
    // vim file.txt 包含空格，是命令不是路径
    assert!(!is_likely_path("vim file.txt"));
    assert!(!is_likely_path("node server.js"));
    
    // 单独的盘符（Windows）
    assert!(!is_likely_path("C:"));
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败（函数不存在）
2. Green：实现 `is_likely_path` 函数
3. Refactor：优化逻辑，提取常量

---

### 模块 2：路径简化（`simplify_path`）

#### 测试用例 2.1：短路径
```rust
#[test]
fn test_short_path() {
    assert_eq!(simplify_path("/home"), "/home");
    assert_eq!(simplify_path("/home/user"), "/home/user");
    assert_eq!(simplify_path("/home/user/project"), "/home/user/project");
}
```

#### 测试用例 2.2：长路径
```rust
#[test]
fn test_long_path() {
    assert_eq!(
        simplify_path("/home/user/project/src/main.rs"),
        ".../project/src/main.rs"
    );
    assert_eq!(
        simplify_path("C:\\Users\\test\\Documents\\work\\project\\file.txt"),
        ".../work/project/file.txt"
    );
}
```

#### 测试用例 2.3：边界情况
```rust
#[test]
fn test_edge_cases_path() {
    // 根路径
    assert_eq!(simplify_path("/"), "/");
    
    // Windows 盘符
    assert_eq!(simplify_path("C:\\"), "C:");
    
    // 空路径
    assert_eq!(simplify_path(""), "");
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：实现 `simplify_path` 函数
3. Refactor：使用 `std::path::MAIN_SEPARATOR`

---

### 模块 3：Git 分支读取（`read_git_branch`）

#### 测试用例 3.1：正常分支
```rust
#[test]
fn test_normal_branch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    assert_eq!(read_git_branch(&git_dir), Some("main".to_string()));
    
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/develop\n").unwrap();
    assert_eq!(read_git_branch(&git_dir), Some("develop".to_string()));
}
```

#### 测试用例 3.2：detached HEAD
```rust
#[test]
fn test_detached_head() {
    let temp_dir = tempfile::tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    
    fs::write(git_dir.join("HEAD"), "a1b2c3d4e5f6g7h8i9j0\n").unwrap();
    assert_eq!(read_git_branch(&git_dir), Some("HEAD".to_string()));
}
```

#### 测试用例 3.3：文件不存在
```rust
#[test]
fn test_head_not_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    
    assert_eq!(read_git_branch(&git_dir), None);
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：实现 `read_git_branch` 函数（已存在，可能需要修改）
3. Refactor：优化错误处理

---

### 模块 4：Git 仓库检测（`find_git_dir`）

#### 测试用例 4.1：在 git 仓库中
```rust
#[test]
fn test_in_git_repo() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path().join("project");
    let git_dir = project_dir.join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    
    let found = find_git_dir(&project_dir).unwrap();
    assert_eq!(found, git_dir);
}
```

#### 测试用例 4.2：在子目录中
```rust
#[test]
fn test_in_subdirectory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_dir = temp_dir.path().join("project");
    let git_dir = project_dir.join(".git");
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(&git_dir).unwrap();
    
    let found = find_git_dir(&src_dir).unwrap();
    assert_eq!(found, git_dir);
}
```

#### 测试用例 4.3：不在 git 仓库中
```rust
#[test]
fn test_not_in_git_repo() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    assert_eq!(find_git_dir(temp_dir.path()), None);
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：实现 `find_git_dir` 函数（已存在，可能需要修改）
3. Refactor：优化查找逻辑

---

### 模块 5：Git 状态获取（`get_git_status`）- **需要改进**

#### 测试用例 5.1：clean 状态
```rust
#[test]
fn test_git_status_clean() {
    let temp_dir = tempfile::tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();
    
    // 创建空的 index 文件
    fs::write(git_dir.join("index"), "").unwrap();
    
    // 当前实现总是返回 "✓"，需要改进
    assert_eq!(get_git_status(&git_dir), Some("✓".to_string()));
}
```

#### 测试用例 5.2：dirty 状态（**新需求**）
```rust
#[test]
fn test_git_status_dirty() {
    // TODO: 实现真实的 git 状态检查
    // 需要运行 `git status --porcelain` 或检查 index 时间戳
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：改进 `get_git_status` 函数，实现真实状态检查
3. Refactor：优化性能

---

### 模块 6：Shell 命令过滤（`is_shell_command`）

#### 测试用例 6.1：常见 shell
```rust
#[test]
fn test_common_shells() {
    assert!(is_shell_command("bash"));
    assert!(is_shell_command("zsh"));
    assert!(is_shell_command("fish"));
    assert!(is_shell_command("sh"));
    assert!(is_shell_command("pwsh"));
    assert!(is_shell_command("powershell"));
    assert!(is_shell_command("cmd"));
}
```

#### 测试用例 6.2：非 shell 命令
```rust
#[test]
fn test_non_shell_commands() {
    assert!(!is_shell_command("vim"));
    assert!(!is_shell_command("node"));
    assert!(!is_shell_command("cargo"));
    assert!(!is_shell_command("git"));
}
```

#### 测试用例 6.3：带路径的命令
```rust
#[test]
fn test_commands_with_path() {
    assert!(is_shell_command("/bin/bash"));
    assert!(is_shell_command("C:\\Windows\\System32\\cmd.exe"));
    assert!(!is_shell_command("/usr/bin/vim"));
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：实现 `is_shell_command` 函数（已存在，可能需要修改）
3. Refactor：提取 shell 列表为常量

---

### 模块 7：路径获取回退链（`get_current_working_dir`）

#### 测试用例 7.1：OSC 7 优先
```rust
#[test]
fn test_osc7_priority() {
    // 模拟 OSC 7 已设置
    // 模拟进程检查可用
    // 模拟 initial_cwd 可用
    
    // 应该返回 OSC 7 的路径
}
```

#### 测试用例 7.2：进程检查回退
```rust
#[test]
fn test_process_fallback() {
    // 模拟 OSC 7 未设置
    // 模拟进程检查可用
    // 模拟 initial_cwd 可用
    
    // 应该返回进程检查的路径
}
```

#### 测试用例 7.3：initial_cwd 最终回退
```rust
#[test]
fn test_initial_cwd_fallback() {
    // 模拟 OSC 7 未设置
    // 模拟进程检查失败
    // 模拟 initial_cwd 可用
    
    // 应该返回 initial_cwd
}
```

#### 测试用例 7.4：全部失败
```rust
#[test]
fn test_all_failed() {
    // 模拟 OSC 7 未设置
    // 模拟进程检查失败
    // 模拟 initial_cwd 为 None
    
    // 应该返回 None
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：实现回退链逻辑（已实现，需要验证）
3. Refactor：优化测试模拟

---

### 模块 8：缓存机制

#### 测试用例 8.1：缓存命中
```rust
#[test]
fn test_cache_hit() {
    // 第一次调用，缓存未命中
    let result1 = get_extra_info(pane, cwd, 1000);
    
    // 立即第二次调用，缓存命中
    let result2 = get_extra_info(pane, cwd, 1000);
    
    assert_eq!(result1, result2);
}
```

#### 测试用例 8.2：缓存过期
```rust
#[test]
fn test_cache_expired() {
    // 第一次调用
    let result1 = get_extra_info(pane, cwd, 100);
    
    // 等待 200ms
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // 第二次调用，缓存已过期
    let result2 = get_extra_info(pane, cwd, 100);
    
    // 可能不同（如果数据变化了）
}
```

#### 测试用例 8.3：缓存键变化
```rust
#[test]
fn test_cache_key_changed() {
    // 调用 get_extra_info(pane1, cwd1)
    // 调用 get_extra_info(pane1, cwd2)
    // 调用 get_extra_info(pane2, cwd1)
    
    // 应该是三个不同的缓存条目
}
```

**TDD 步骤**：
1. Red：写上述测试，运行 → 失败
2. Green：实现缓存机制（已存在，需要验证）
3. Refactor：优化缓存键设计

---

### 模块 9：集成测试

#### 测试用例 9.1：首次打开终端（开箱即显）
```rust
#[test]
fn test_immediate_display() {
    // 创建 pane，设置 initial_cwd
    // OSC 7 未发送
    // 进程检查可能失败
    
    // 渲染 tab bar
    // 验证：路径显示（使用 initial_cwd）
    // 验证：git 分支显示（如果 initial_cwd 在 git 仓库中）
}
```

#### 测试用例 9.2：用户运行 cd
```rust
#[test]
fn test_cd_updates_path() {
    // 初始状态：路径=/path1, git=main
    
    // 模拟用户运行 cd /path2
    // 发送 OSC 7
    
    // 验证：路径更新为 /path2
    // 验证：git 分支更新（如果 /path2 在不同的 git 仓库）
}
```

#### 测试用例 9.3：用户切换 git 分支
```rust
#[test]
fn test_git_checkout_updates_branch() {
    // 初始状态：git=main
    
    // 模拟用户运行 git checkout develop
    // .git/HEAD 内容改变
    
    // 验证：git 分支更新为 develop
}
```

#### 测试用例 9.4：标题变化
```rust
#[test]
fn test_title_change() {
    // 初始状态：标题="bash", 显示路径行
    
    // 模拟程序设置标题为 "claude"
    
    // 验证：标题变为 "claude"
    // 验证：路径行仍然显示（因为 "claude" 不是路径）
}
```

**TDD 步骤**：
1. Red：写上述集成测试，运行 → 失败
2. Green：实现完整功能
3. Refactor：优化整体架构

---

## 📊 测试覆盖率目标

| 模块 | 目标覆盖率 | 优先级 |
|------|-----------|--------|
| `is_likely_path` | 100% | 高 |
| `simplify_path` | 100% | 高 |
| `read_git_branch` | 100% | 高 |
| `find_git_dir` | 100% | 高 |
| `get_git_status` | 100% | 中 |
| `is_shell_command` | 100% | 中 |
| `get_current_working_dir` | 100% | 高 |
| 缓存机制 | 100% | 中 |
| 集成测试 | 关键场景 100% | 高 |

---

## 🔄 TDD 开发顺序

### 阶段 1：核心工具函数（优先级：高）
1. `is_likely_path` - 标题路径判断
2. `simplify_path` - 路径简化
3. `read_git_branch` - Git 分支读取
4. `find_git_dir` - Git 仓库检测

### 阶段 2：辅助函数（优先级：中）
5. `get_git_status` - Git 状态获取（改进）
6. `is_shell_command` - Shell 命令过滤

### 阶段 3：核心逻辑（优先级：高）
7. `get_current_working_dir` - 路径获取回退链
8. 缓存机制验证

### 阶段 4：集成测试（优先级：高）
9. 开箱即显测试
10. 动态更新测试
11. 边界情况测试

---

## 📝 TDD 开发规范

### 1. 测试命名规范
```rust
#[test]
fn test_<功能>_<场景>() {
    // 测试代码
}
```

### 2. 测试结构（AAA 模式）
```rust
#[test]
fn test_example() {
    // Arrange：准备测试数据
    let input = "test";
    
    // Act：执行被测试函数
    let result = function_under_test(input);
    
    // Assert：验证结果
    assert_eq!(result, expected);
}
```

### 3. 测试文件组织
```
wezterm-gui/src/termwindow/
├── tab_extra_info.rs          # 实现代码
└── tab_extra_info_test.rs     # 测试代码（或使用 #[cfg(test)] mod tests）
```

### 4. Mock 和 Stub
对于外部依赖（文件系统、进程等），使用：
- `tempfile` crate 创建临时目录
- Mock 对象模拟外部依赖
- 集成测试使用真实环境

---

## ✅ 验收标准

完成 TDD 开发后，必须满足：

1. **所有测试通过**：`cargo test` 100% 通过
2. **测试覆盖率达标**：核心模块 100%，辅助模块 ≥ 90%
3. **功能验证**：
   - ✅ 开箱即显：首次打开终端立即显示路径和 git 分支
   - ✅ 动态更新：用户操作后立即更新
   - ✅ 边界情况：所有边界情况正确处理
4. **性能验证**：
   - 路径获取：< 10ms
   - Git 分支读取：< 5ms
   - 整体渲染：< 50ms
5. **跨平台验证**：
   - Windows、macOS、Linux 全部通过测试

---

## 🚀 执行计划

### 第 1 天：阶段 1（核心工具函数）
- 编写 `is_likely_path` 测试 → 实现 → 重构
- 编写 `simplify_path` 测试 → 实现 → 重构
- 编写 `read_git_branch` 测试 → 实现 → 重构
- 编写 `find_git_dir` 测试 → 实现 → 重构

### 第 2 天：阶段 2（辅助函数）
- 编写 `get_git_status` 测试 → 改进 → 重构
- 编写 `is_shell_command` 测试 → 实现 → 重构

### 第 3 天：阶段 3（核心逻辑）
- 编写 `get_current_working_dir` 测试 → 验证 → 重构
- 编写缓存机制测试 → 验证 → 重构

### 第 4 天：阶段 4（集成测试）
- 编写开箱即显测试 → 验证
- 编写动态更新测试 → 验证
- 编写边界情况测试 → 验证

### 第 5 天：验收和优化
- 运行所有测试，确保 100% 通过
- 测量测试覆盖率
- 性能测试和优化
- 跨平台验证

---

## 📌 关键提醒

1. **严格遵循 Red-Green-Refactor 循环**
2. **每次只写一个测试**，让它失败，然后写最少代码让它通过
3. **不要跳过测试**，即使看起来很简单
4. **重构时保持测试通过**
5. **集成测试使用真实环境**，单元测试使用 Mock

---

**准备好开始 TDD 开发了吗？**
