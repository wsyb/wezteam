//! Tab 信息面板：为垂直 Tab 栏提供路径、Git 分支、当前命令。
//!
//! 数据流：
//!   标题含路径分隔符 → 从标题提取倒序路径 → 正序化查 git
//!   标题不含分隔符 → 从 OSC 7 取正序 CWD → 倒序化显示 + 正序查 git
//!
//! 显示规则：
//!   路径：倒序显示（与 tab 标题格式一致），仅当标题非路径时显示
//!   Git：正序路径查 .git/HEAD，仅当在 git 仓库中时显示
//!   命令：前台进程，仅当非 shell 时显示

use mux::pane::CachePolicy;
use std::path::Path;

// ============================================================
// 公共 API
// ============================================================

/// Tab 面板额外信息
pub struct TabExtraInfo {
    /// 倒序路径（与 tab 标题格式一致，仅当标题非路径时显示）
    pub reversed_path: Option<String>,
    /// Git 分支名
    pub git_branch: Option<String>,
    /// 当前命令（已过滤 shell 本身）
    pub current_command: Option<String>,
}

/// 获取 Tab 额外信息
///
/// 入口函数，数据层与 UI 层的唯一接口。
pub fn get_tab_extra_info(pane: &dyn mux::pane::Pane, tab_title: &str) -> TabExtraInfo {
    let title_path = extract_reversed_path_from_title(tab_title);
    let osc7_cwd = get_pane_cwd(pane);
    let title_is_path = title_path.is_some();

    // 倒序路径：仅当标题非路径时显示，从 OSC 7 CWD 倒序化
    let reversed_path = if !title_is_path {
        osc7_cwd.as_ref().map(|p| path_to_reversed(p))
    } else {
        None
    };

    // 正序 CWD：标题路径正序化优先，其次 OSC 7
    let normal_cwd = if title_is_path {
        title_path.as_ref().map(|p| reversed_to_normal(p))
    } else {
        osc7_cwd
    };

    TabExtraInfo {
        reversed_path,
        git_branch: normal_cwd.as_ref().and_then(|p| get_git_branch(p)),
        current_command: get_current_command(pane),
    }
}

// ============================================================
// 路径：倒序 ↔ 正序
// ============================================================

/// 正序路径 → 倒序路径
///
/// "D:\work\wezteam" → "wezteam\work\D:"
/// "/home/user/project" → "project/user/home"
fn path_to_reversed(path: &str) -> String {
    reverse_path_components(path)
}

/// 倒序路径 → 正序路径
///
/// "wezteam\work\D:" → "D:\work\wezteam"
/// "project/user/home" → "/home/user/project"（Unix 不存在倒序，但逻辑等价）
fn reversed_to_normal(reversed: &str) -> String {
    reverse_path_components(reversed)
}

/// 路径组件反转（倒序↔正序互为逆运算，逻辑相同）
fn reverse_path_components(path: &str) -> String {
    let trimmed = path.trim_end_matches(|c| c == '\\' || c == '/');
    let sep = if trimmed.contains('\\') { '\\' } else { '/' };
    let parts: Vec<&str> = trimmed.split(sep).filter(|s| !s.is_empty()).rev().collect();
    parts.join(&sep.to_string())
}

/// 从 tab 标题提取倒序路径
///
/// 标题含路径分隔符（/ 或 \）时视为路径，去除编号前缀后返回。
/// 标题不含分隔符时返回 None。
fn extract_reversed_path_from_title(title: &str) -> Option<String> {
    let clean = strip_tab_index(title);
    if !clean.contains('/') && !clean.contains('\\') {
        return None;
    }
    Some(clean.to_string())
}

/// 去除 tab 编号前缀
///
/// "1: wezteam\work\D:" → "wezteam\work\D:"
/// "claude" → "claude"
fn strip_tab_index(title: &str) -> &str {
    title.splitn(2, ": ").last().unwrap_or(title)
}

/// 获取 pane 当前工作目录（OSC 7，正序）
fn get_pane_cwd(pane: &dyn mux::pane::Pane) -> Option<String> {
    pane.get_current_working_dir(CachePolicy::FetchImmediate)
        .and_then(|url| url.to_file_path().ok())
        .and_then(|path| path.into_os_string().into_string().ok())
}

// ============================================================
// Git 分支
// ============================================================

/// 从正序路径获取 git 分支名
fn get_git_branch(normal_path: &str) -> Option<String> {
    let git_dir = find_git_dir(Path::new(normal_path))?;
    read_git_branch(&git_dir)
}

/// 从起始路径向上查找 .git 目录
fn find_git_dir(start: &Path) -> Option<std::path::PathBuf> {
    let mut current = start;
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(git_dir);
        }
        current = current.parent()?;
    }
}

/// 读取 .git/HEAD 获取分支名
///
/// 正常分支：ref: refs/heads/main → "main"
/// Detached HEAD：abc1234 → "HEAD"
fn read_git_branch(git_dir: &Path) -> Option<String> {
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        Some(branch.trim().to_string())
    } else {
        Some("HEAD".to_string())
    }
}

// ============================================================
// 当前命令
// ============================================================

const SHELLS: &[&str] = &["bash", "zsh", "fish", "sh", "pwsh", "powershell", "cmd"];

/// 获取前台进程命令（过滤 shell 本身）
fn get_current_command(pane: &dyn mux::pane::Pane) -> Option<String> {
    let info = pane.get_foreground_process_info(CachePolicy::FetchImmediate)?;
    if info.argv.is_empty() {
        return None;
    }
    let name = Path::new(&info.argv[0]).file_name()?.to_str()?.to_string();
    if is_shell(&name) {
        return None;
    }
    Some(format_command(&name, &info.argv[1..]))
}

/// 判断进程名是否为 shell
fn is_shell(name: &str) -> bool {
    SHELLS.iter().any(|s| name.eq_ignore_ascii_case(s))
}

/// 格式化命令行
fn format_command(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, args.join(" "))
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- path_to_reversed ----

    #[test]
    fn test_path_to_reversed_windows() {
        assert_eq!(path_to_reversed("D:\\work\\wezteam"), "wezteam\\work\\D:");
    }

    #[test]
    fn test_path_to_reversed_windows_trailing_slash() {
        assert_eq!(path_to_reversed("D:\\work\\wezteam\\"), "wezteam\\work\\D:");
    }

    #[test]
    fn test_path_to_reversed_unix() {
        assert_eq!(path_to_reversed("/home/user/project"), "project/user/home");
    }

    #[test]
    fn test_path_to_reversed_drive_root() {
        assert_eq!(path_to_reversed("D:\\"), "D:");
    }

    #[test]
    fn test_path_to_reversed_single_component() {
        assert_eq!(path_to_reversed("wezteam"), "wezteam");
    }

    // ---- reversed_to_normal ----

    #[test]
    fn test_reversed_to_normal_windows() {
        assert_eq!(reversed_to_normal("wezteam\\work\\D:"), "D:\\work\\wezteam");
    }

    #[test]
    fn test_reversed_to_normal_unix() {
        assert_eq!(reversed_to_normal("project/user/home"), "home/user/project");
    }

    #[test]
    fn test_reversed_to_normal_roundtrip() {
        let original = "D:\\work\\wezteam";
        assert_eq!(reversed_to_normal(&path_to_reversed(original)), original);
    }

    #[test]
    fn test_path_to_reversed_roundtrip() {
        let original = "wezteam\\work\\D:";
        assert_eq!(path_to_reversed(&reversed_to_normal(original)), original);
    }

    // ---- strip_tab_index ----

    #[test]
    fn test_strip_tab_index_with_prefix() {
        assert_eq!(strip_tab_index("1: wezteam\\work\\D:"), "wezteam\\work\\D:");
    }

    #[test]
    fn test_strip_tab_index_without_prefix() {
        assert_eq!(strip_tab_index("claude"), "claude");
    }

    #[test]
    fn test_strip_tab_index_double_digit() {
        assert_eq!(strip_tab_index("12: some\\path"), "some\\path");
    }

    #[test]
    fn test_strip_tab_index_no_space_after_colon() {
        // "1:abc" — 没有 ": " 模式，整体返回
        assert_eq!(strip_tab_index("1:abc"), "1:abc");
    }

    // ---- extract_reversed_path_from_title ----

    #[test]
    fn test_extract_path_from_reversed_title() {
        assert_eq!(
            extract_reversed_path_from_title("1: wezteam\\work\\D:"),
            Some("wezteam\\work\\D:".to_string())
        );
    }

    #[test]
    fn test_extract_path_from_normal_title() {
        assert_eq!(
            extract_reversed_path_from_title("1: D:\\work\\wezteam"),
            Some("D:\\work\\wezteam".to_string())
        );
    }

    #[test]
    fn test_extract_path_from_program_title() {
        assert_eq!(extract_reversed_path_from_title("1: claude"), None);
    }

    #[test]
    fn test_extract_path_from_program_no_index() {
        assert_eq!(extract_reversed_path_from_title("vim"), None);
    }

    #[test]
    fn test_extract_path_from_unix_title() {
        assert_eq!(
            extract_reversed_path_from_title("2: /home/user/project"),
            Some("/home/user/project".to_string())
        );
    }

    // ---- is_shell ----

    #[test]
    fn test_is_shell_bash() {
        assert!(is_shell("bash"));
    }

    #[test]
    fn test_is_shell_pwsh() {
        assert!(is_shell("pwsh"));
    }

    #[test]
    fn test_is_shell_case_insensitive() {
        assert!(is_shell("PowerShell"));
        assert!(is_shell("CMD"));
    }

    #[test]
    fn test_is_shell_not_shell() {
        assert!(!is_shell("claude"));
        assert!(!is_shell("vim"));
        assert!(!is_shell("node"));
    }

    // ---- format_command ----

    #[test]
    fn test_format_command_no_args() {
        assert_eq!(format_command("vim", &[]), "vim");
    }

    #[test]
    fn test_format_command_with_args() {
        assert_eq!(
            format_command("cargo", &["build".to_string(), "--release".to_string()]),
            "cargo build --release"
        );
    }

    // ---- find_git_dir ----

    #[test]
    fn test_find_git_dir_in_project_root() {
        // 当前目录就是 git 仓库根目录
        let cwd = std::env::current_dir().unwrap();
        let result = find_git_dir(&cwd);
        assert!(result.is_some(), "should find .git in project root");
    }

    #[test]
    fn test_find_git_dir_in_subdirectory() {
        // src 是项目子目录，向上查找应找到 .git
        let cwd = std::env::current_dir().unwrap();
        let src_dir = cwd.join("src");
        if src_dir.exists() {
            let result = find_git_dir(&src_dir);
            assert!(result.is_some(), "should find .git by walking up from src/");
        }
    }

    #[test]
    fn test_find_git_dir_not_found() {
        // 系统根目录不应有 .git
        let result = find_git_dir(Path::new("C:\\"));
        assert!(result.is_none(), "should not find .git in drive root");
    }

    // ---- read_git_branch ----

    #[test]
    fn test_read_git_branch_in_project() {
        let cwd = std::env::current_dir().unwrap();
        let git_dir = find_git_dir(&cwd).expect("should find .git");
        let branch = read_git_branch(&git_dir);
        assert!(branch.is_some(), "should read branch from .git/HEAD");
        // 当前项目在 main 分支
        let name = branch.unwrap();
        assert!(!name.is_empty(), "branch name should not be empty");
    }

    // ---- 集成：extract + reversed_to_normal 联动 ----

    #[test]
    fn test_title_path_to_normal_cwd() {
        // 模拟完整流程：标题是倒序路径 → 提取 → 正序化 → 可用于 git 查找
        let title = "1: wezteam\\work\\D:";
        let reversed = extract_reversed_path_from_title(title).unwrap();
        let normal = reversed_to_normal(&reversed);
        assert_eq!(normal, "D:\\work\\wezteam");
    }

    #[test]
    fn test_title_path_to_normal_cwd_unix() {
        let title = "2: project/user/home";
        let reversed = extract_reversed_path_from_title(title).unwrap();
        let normal = reversed_to_normal(&reversed);
        assert_eq!(normal, "home/user/project");
    }

    // ---- 边界情况 ----

    #[test]
    fn test_empty_title() {
        assert_eq!(extract_reversed_path_from_title(""), None);
    }

    #[test]
    fn test_path_to_reversed_empty() {
        assert_eq!(path_to_reversed(""), "");
    }

    #[test]
    fn test_reversed_to_normal_empty() {
        assert_eq!(reversed_to_normal(""), "");
    }

    #[test]
    fn test_strip_tab_index_empty() {
        assert_eq!(strip_tab_index(""), "");
    }

    #[test]
    fn test_format_command_single_arg() {
        assert_eq!(format_command("git", &["status".to_string()]), "git status");
    }
}
