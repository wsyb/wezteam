use mux::pane::CachePolicy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type PaneId = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    pane_id: PaneId,
    cwd: String,
}

struct CacheEntry {
    git_branch: Option<String>,
    last_command: Option<String>,
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
    cache_duration_ms: u64,
) -> (Option<String>, Option<String>) {
    let pane_id = pane.pane_id();
    let cache_duration = Duration::from_millis(cache_duration_ms);
    
    let cwd = match pane.get_current_working_dir(CachePolicy::AllowStale) {
        Some(url) => url.to_string(),
        None => return (None, None),
    };
    
    let key = CacheKey { pane_id, cwd: cwd.clone() };
    
    {
        let cache = CACHE.lock().unwrap();
        if let Some(entry) = cache.entries.get(&key) {
            if entry.timestamp.elapsed() < cache_duration {
                return (entry.git_branch.clone(), entry.last_command.clone());
            }
        }
    }
    
    let git_branch = fetch_git_branch(pane);
    let last_command = fetch_last_command(pane);
    
    {
        let mut cache = CACHE.lock().unwrap();
        cache.cache_duration = cache_duration;
        cache.entries.insert(key, CacheEntry {
            git_branch: git_branch.clone(),
            last_command: last_command.clone(),
            timestamp: Instant::now(),
        });
    }
    
    (git_branch, last_command)
}

fn fetch_last_command(pane: &dyn mux::pane::Pane) -> Option<String> {
    let proc_info = pane.get_foreground_process_info(CachePolicy::AllowStale)?;
    
    if proc_info.argv.is_empty() {
        return None;
    }
    
    let cmd = &proc_info.argv[0];
    let cmd_name = Path::new(cmd).file_name()?.to_str()?.to_string();
    
    if is_shell_command(&cmd_name) {
        return None;
    }
    
    let full_cmd = if proc_info.argv.len() > 1 {
        format!("{} {}", cmd_name, proc_info.argv[1..].join(" "))
    } else {
        cmd_name
    };
    
    Some(truncate_command(&full_cmd, 20))
}

fn fetch_git_branch(pane: &dyn mux::pane::Pane) -> Option<String> {
    let cwd = pane.get_current_working_dir(CachePolicy::AllowStale)?;
    
    if cwd.scheme() != "file" {
        return None;
    }
    
    let path = cwd.to_file_path().ok()?;
    let git_dir = find_git_dir(&path)?;
    
    let branch = read_git_branch(&git_dir)?;
    let status = get_git_status(&git_dir)?;
    
    Some(format!("git:{} {}", branch, status))
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
