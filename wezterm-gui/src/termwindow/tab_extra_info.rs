use mux::pane::CachePolicy;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

fn log_debug(msg: &str) {
    let log_file = std::env::temp_dir().join("wezterm_extra_info.log");
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        let elapsed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = elapsed.as_secs() % 86400;
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        let millis = elapsed.subsec_millis();
        let _ = writeln!(file, "[{:02}:{:02}:{:02}.{:03}] {}", hours, mins, secs, millis, msg);
    }
}

type PaneId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    pane_id: PaneId,
    cwd: String,
}

struct CacheEntry {
    git_branch: Option<String>,
    last_command: Option<String>,
    last_non_shell_command: Option<String>,
    timestamp: Instant,
}

struct GlobalCache {
    entries: HashMap<CacheKey, CacheEntry>,
    cache_duration: Duration,
}

impl GlobalCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            cache_duration: Duration::from_millis(1000),
        }
    }
}

lazy_static::lazy_static! {
    static ref CACHE: Arc<Mutex<GlobalCache>> = Arc::new(Mutex::new(GlobalCache::new()));
}

pub fn get_extra_info(
    pane: &dyn mux::pane::Pane,
    pane_title: &str,
    cache_duration_ms: u64,
) -> (Option<String>, Option<String>) {
    let pane_id = pane.pane_id();
    let cache_duration = Duration::from_millis(cache_duration_ms);
    
    // Use pane_title as cwd (more reliable than get_current_working_dir)
    log_debug(&format!("Pane {} title: {}", pane_id, pane_title));
    
    let cwd = pane_title.to_string();
    
    let key = CacheKey { pane_id, cwd: cwd.clone() };
    
    let git_branch = fetch_git_branch_from_path(&cwd);
    let current_cmd = fetch_last_command(pane);

    // When pane title changes (e.g. shell → node), cache key changes.
    // But the old cached last_non_shell_command (e.g. "npm run dev") leaks through.
    // Fix: only use current_cmd, never fall back to stale cached command.
    let result_cmd = current_cmd.clone();

    {
        let mut cache = CACHE.lock().unwrap();
        cache.cache_duration = cache_duration;
        cache.entries.insert(key, CacheEntry {
            git_branch: git_branch.clone(),
            last_command: current_cmd.clone(),
            last_non_shell_command: result_cmd.clone(),
            timestamp: Instant::now(),
        });
    }

    log_debug(&format!("Pane {} result: git={:?}, cmd={:?}", pane_id, git_branch, result_cmd));
    (git_branch, result_cmd)
}

fn fetch_last_command(pane: &dyn mux::pane::Pane) -> Option<String> {
    let proc_info = match pane.get_foreground_process_info(CachePolicy::FetchImmediate) {
        Some(info) => info,
        None => {
            log_debug("fetch_last_command: failed to get proc_info");
            return None;
        }
    };
    
    if proc_info.argv.is_empty() {
        log_debug("fetch_last_command: argv is empty");
        return None;
    }
    
    let cmd = &proc_info.argv[0];
    let cmd_name = match Path::new(cmd).file_name() {
        Some(name) => match name.to_str() {
            Some(s) => s.to_string(),
            None => {
                log_debug(&format!("fetch_last_command: cmd_name not utf8: {:?}", name));
                return None;
            }
        }
        None => {
            log_debug(&format!("fetch_last_command: no filename in: {}", cmd));
            return None;
        }
    };

    if is_shell_command(&cmd_name) {
        log_debug(&format!("fetch_last_command: {} is shell, skipping", cmd_name));
        return None;
    }

    let full_cmd = if proc_info.argv.len() > 1 {
        format!("{} {}", cmd_name, proc_info.argv[1..].join(" "))
    } else {
        cmd_name.clone()
    };

    // 128 chars: show full command without truncation in normal cases
    let result = truncate_command(&full_cmd, 128);
    log_debug(&format!("fetch_last_command: {} -> {}", cmd_name, result));
    Some(result)
}

fn fetch_git_branch_from_path(path_str: &str) -> Option<String> {
    // NOTE: path_str 是倒序的（如 "src\wezteam\work\D:"），需要反转后才能正确向上查找 .git
    // 反转逻辑只用于内部查找，不影响显示。如果修改此逻辑，请测试 Windows/macOS/Linux 三端。

    // Parse path from pane title
    // The title might be reversed like "wezteam\work\D:" instead of "D:\work\wezteam"
    let path_str = if path_str.contains(':') && path_str.chars().last() == Some(':') {
        // Reversed path: "wezteam\work\D:" -> "D:\work\wezteam"
        let parts: Vec<&str> = path_str.split('\\').rev().collect();
        parts.join("\\")
    } else {
        path_str.to_string()
    };
    
    log_debug(&format!("fetch_git_branch_from_path: normalized path: {}", path_str));
    
    let path = Path::new(&path_str);
    
    let git_dir = match find_git_dir(path) {
        Some(dir) => dir,
        None => {
            log_debug(&format!("fetch_git_branch_from_path: no .git in {:?}", path));
            return None;
        }
    };
    
    let branch = match read_git_branch(&git_dir) {
        Some(b) => b,
        None => {
            log_debug(&format!("fetch_git_branch_from_path: failed to read branch from {:?}", git_dir));
            return None;
        }
    };
    
    let status = match get_git_status(&git_dir) {
        Some(s) => s,
        None => {
            log_debug(&format!("fetch_git_branch_from_path: failed to get status from {:?}", git_dir));
            return None;
        }
    };
    
    let result = format!("git:{} {}", branch, status);
    log_debug(&format!("fetch_git_branch_from_path: {:?}", result));
    Some(result)
}

fn is_shell_command(cmd: &str) -> bool {
    let shells = ["bash", "zsh", "fish", "sh", "pwsh", "powershell", "cmd"];
    shells.iter().any(|&s| cmd.contains(s))
}

fn truncate_command(cmd: &str, max_len: usize) -> String {
    if cmd.len() <= max_len {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max_len.saturating_sub(3)])
    }
}

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

fn read_git_branch(git_dir: &Path) -> Option<String> {
    let head_path = git_dir.join("HEAD");
    let head_content = std::fs::read_to_string(&head_path).ok()?;
    
    if head_content.starts_with("ref: refs/heads/") {
        Some(head_content[16..].trim().to_string())
    } else {
        Some("HEAD".to_string())
    }
}

fn get_git_status(git_dir: &Path) -> Option<String> {
    let index_path = git_dir.join("index");
    if !index_path.exists() {
        return Some("✓".to_string());
    }
    
    Some("✓".to_string())
}
