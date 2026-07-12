//! Tab info panel: provides path, git branch, and current command for vertical tab bar.
//!
//! Data flow:
//!   Title contains path separator -> extract reversed path from title -> normalize for git lookup
//!   Title has no separator -> get normal CWD from OSC 7 -> reverse for display + use as-is for git
//!
//! Display rules:
//!   Path: shown in reversed order (matching tab title format), only when title is not a path
//!   Git: look up .git/HEAD using normal path, only shown when inside a git repo
//!   Command: foreground process, only shown when not a shell

use mux::pane::CachePolicy;
use std::path::Path;
use url::Url;

// ============================================================
// Public API
// ============================================================

/// Extra info for a tab in the vertical tab bar
pub struct TabExtraInfo {
    /// Reversed path (matching tab title format, only shown when title is not a path)
    pub reversed_path: Option<String>,
    /// Git branch name
    pub git_branch: Option<String>,
    /// Current command (shell processes filtered out)
    pub current_command: Option<String>,
}

/// Get extra info for a tab
///
/// Entry point and the only interface between the data layer and the UI layer.
pub fn get_tab_extra_info(pane: &dyn mux::pane::Pane, tab_title: &str) -> TabExtraInfo {
    // Always get the REAL CWD from the process — never trust the title for CWD.
    // The title is set by the shell prompt and doesn't update when you `cd`.
    let real_cwd = get_pane_cwd(pane);

    let title_path = extract_reversed_path_from_title(tab_title);
    let title_is_path = title_path.is_some();

    // Reversed path: only shown when title is not a path, derived from real CWD
    let reversed_path = if !title_is_path {
        real_cwd.as_ref().map(|p| path_to_reversed(p))
    } else {
        None
    };

    TabExtraInfo {
        reversed_path,
        git_branch: real_cwd.as_ref().and_then(|p| get_git_branch(p)),
        current_command: get_current_command(pane),
    }
}

// ============================================================
// Path: reversed <-> normal
// ============================================================

/// Normal path -> reversed path
///
/// "D:\work\project" -> "project\work\D:"
/// "/home/user/project" -> "project/user/home"
fn path_to_reversed(path: &str) -> String {
    reverse_path_components(path)
}

/// Reversed path -> normal path
///
/// "project\work\D:" -> "D:\work\project"
/// "project/user/home" -> "/home/user/project" (Unix reversal is logically equivalent)
fn reversed_to_normal(reversed: &str) -> String {
    reverse_path_components(reversed)
}

/// Reverse path components (reversed<->normal are inverse operations, same logic)
fn reverse_path_components(path: &str) -> String {
    let trimmed = path.trim_end_matches(|c| c == '\\' || c == '/');
    let sep = if trimmed.contains('\\') { '\\' } else { '/' };
    let parts: Vec<&str> = trimmed.split(sep).filter(|s| !s.is_empty()).rev().collect();
    parts.join(&sep.to_string())
}

/// Extract reversed path from tab title
///
/// When the title contains a path separator (/ or \), it is treated as a path.
/// The tab index prefix is stripped before returning.
/// Returns None when the title contains no separator.
fn extract_reversed_path_from_title(title: &str) -> Option<String> {
    let clean = strip_tab_index(title);
    if !clean.contains('/') && !clean.contains('\\') {
        return None;
    }
    Some(clean.to_string())
}

/// Strip tab index prefix
///
/// "1: project\work\D:" -> "project\work\D:"
/// "claude" -> "claude"
fn strip_tab_index(title: &str) -> &str {
    title.splitn(2, ": ").last().unwrap_or(title)
}

/// Get pane current working directory (normal order)
///
/// Two-layer fallback:
///   1. OSC 7 URL → path string (handles file://hostname/path from zsh)
///   2. Foreground process CWD (lightweight /proc/<pid>/cwd read)
fn get_pane_cwd(pane: &dyn mux::pane::Pane) -> Option<String> {
    let osc7_cwd = pane
        .get_current_working_dir(CachePolicy::FetchImmediate)
        .and_then(|url| url_to_path_string(&url));

    let proc_cwd = pane
        .get_foreground_process_cwd(CachePolicy::FetchImmediate)
        .and_then(|path| {
            let s = path.into_os_string().into_string().ok()?;
            if s.is_empty() { None } else { Some(s) }
        });

    osc7_cwd.or(proc_cwd)
}

/// Convert a file:// URL to a filesystem path string.
///
/// Standard `Url::to_file_path()` rejects `file://hostname/path` (zsh sends this).
/// This function falls back to manually extracting the path component when
/// the standard conversion fails due to a non-empty host.
fn url_to_path_string(url: &Url) -> Option<String> {
    if let Ok(path) = url.to_file_path() {
        return path.into_os_string().into_string().ok();
    }
    if url.scheme() == "file" {
        let path_str = url.path();
        if path_str != "/" && !path_str.is_empty() {
            return Path::new(path_str)
                .to_path_buf()
                .into_os_string()
                .into_string()
                .ok();
        }
    }
    None
}

// ============================================================
// Git branch
// ============================================================

/// Get git branch name from a normal (non-reversed) path
fn get_git_branch(normal_path: &str) -> Option<String> {
    let git_dir = find_git_dir(Path::new(normal_path))?;
    read_git_branch(&git_dir)
}

/// Walk up from start path to find .git directory
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

/// Read branch name from .git/HEAD
///
/// Normal branch: ref: refs/heads/main -> "main"
/// Detached HEAD: abc1234 -> "HEAD"
fn read_git_branch(git_dir: &Path) -> Option<String> {
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    if let Some(branch) = head.strip_prefix("ref: refs/heads/") {
        Some(branch.trim().to_string())
    } else {
        Some("HEAD".to_string())
    }
}

// ============================================================
// Current command
// ============================================================

const SHELLS: &[&str] = &["bash", "zsh", "fish", "sh", "pwsh", "powershell", "cmd"];

/// Get foreground process command (shell processes filtered out)
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

/// Check if a process name is a shell
fn is_shell(name: &str) -> bool {
    SHELLS.iter().any(|s| name.eq_ignore_ascii_case(s))
}

/// Format command line display
fn format_command(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, args.join(" "))
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- path_to_reversed ----

    #[test]
    fn test_path_to_reversed_windows() {
        assert_eq!(path_to_reversed("D:\\work\\project"), "project\\work\\D:");
    }

    #[test]
    fn test_path_to_reversed_windows_trailing_slash() {
        assert_eq!(path_to_reversed("D:\\work\\project\\"), "project\\work\\D:");
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
        assert_eq!(path_to_reversed("project"), "project");
    }

    // ---- reversed_to_normal ----

    #[test]
    fn test_reversed_to_normal_windows() {
        assert_eq!(reversed_to_normal("project\\work\\D:"), "D:\\work\\project");
    }

    #[test]
    fn test_reversed_to_normal_unix() {
        assert_eq!(reversed_to_normal("project/user/home"), "home/user/project");
    }

    #[test]
    fn test_reversed_to_normal_roundtrip() {
        let original = "D:\\work\\project";
        assert_eq!(reversed_to_normal(&path_to_reversed(original)), original);
    }

    #[test]
    fn test_path_to_reversed_roundtrip() {
        let original = "project\\work\\D:";
        assert_eq!(path_to_reversed(&reversed_to_normal(original)), original);
    }

    // ---- strip_tab_index ----

    #[test]
    fn test_strip_tab_index_with_prefix() {
        assert_eq!(strip_tab_index("1: project\\work\\D:"), "project\\work\\D:");
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
        // "1:abc" -- no ": " pattern, return as-is
        assert_eq!(strip_tab_index("1:abc"), "1:abc");
    }

    // ---- extract_reversed_path_from_title ----

    #[test]
    fn test_extract_path_from_reversed_title() {
        assert_eq!(
            extract_reversed_path_from_title("1: project\\work\\D:"),
            Some("project\\work\\D:".to_string())
        );
    }

    #[test]
    fn test_extract_path_from_normal_title() {
        assert_eq!(
            extract_reversed_path_from_title("1: D:\\work\\project"),
            Some("D:\\work\\project".to_string())
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
        let cwd = std::env::current_dir().unwrap();
        let result = find_git_dir(&cwd);
        assert!(result.is_some(), "should find .git in project root");
    }

    #[test]
    fn test_find_git_dir_in_subdirectory() {
        let cwd = std::env::current_dir().unwrap();
        let src_dir = cwd.join("src");
        if src_dir.exists() {
            let result = find_git_dir(&src_dir);
            assert!(result.is_some(), "should find .git by walking up from src/");
        }
    }

    #[test]
    fn test_find_git_dir_not_found() {
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
        let name = branch.unwrap();
        assert!(!name.is_empty(), "branch name should not be empty");
    }

    // ---- Integration: extract + reversed_to_normal ----

    #[test]
    fn test_title_path_to_normal_cwd() {
        let title = "1: project\\work\\D:";
        let reversed = extract_reversed_path_from_title(title).unwrap();
        let normal = reversed_to_normal(&reversed);
        assert_eq!(normal, "D:\\work\\project");
    }

    #[test]
    fn test_title_path_to_normal_cwd_unix() {
        let title = "2: project/user/home";
        let reversed = extract_reversed_path_from_title(title).unwrap();
        let normal = reversed_to_normal(&reversed);
        assert_eq!(normal, "home/user/project");
    }

    // ---- Edge cases ----

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

    // ---- url_to_path_string ----

    #[test]
    fn test_url_to_path_no_hostname() {
        let url = url::Url::parse("file:///home/user/project").unwrap();
        assert_eq!(url_to_path_string(&url), Some("/home/user/project".to_string()));
    }

    #[test]
    fn test_url_to_path_with_hostname() {
        let url = url::Url::parse("file://myhost/home/user/project").unwrap();
        assert_eq!(url_to_path_string(&url), Some("/home/user/project".to_string()));
    }

    #[test]
    fn test_url_to_path_with_hostname_windows_style() {
        let url = url::Url::parse("file://myhost/D:/work/project").unwrap();
        let result = url_to_path_string(&url);
        assert!(result.is_some(), "should extract path from file://host/D:/...");
        let path = result.unwrap();
        assert!(path.contains("work"), "path should contain 'work': {}", path);
    }

    #[test]
    fn test_url_to_path_non_file_scheme() {
        let url = url::Url::parse("https://example.com/path").unwrap();
        assert_eq!(url_to_path_string(&url), None);
    }

    #[test]
    fn test_url_to_path_empty_path() {
        let url = url::Url::parse("file://myhost").unwrap();
        assert_eq!(url_to_path_string(&url), None);
    }
}
