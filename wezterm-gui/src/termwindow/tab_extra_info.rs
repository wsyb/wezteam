//! Tab extra information: last command and git branch
//!
//! This module provides functionality to fetch and cache extra information
//! for vertical tab bar display.

use mux::pane::CachePolicy;
use std::path::Path;
use std::time::{Duration, Instant};

/// Cache for tab extra information
pub struct TabExtraInfoCache {
    /// Last command with timestamp
    last_command: Option<(String, Instant)>,
    /// Git branch with timestamp
    git_branch: Option<(String, Instant)>,
    /// Cache duration
    cache_duration: Duration,
}

impl TabExtraInfoCache {
    pub fn new(cache_duration_ms: u64) -> Self {
        Self {
            last_command: None,
            git_branch: None,
            cache_duration: Duration::from_millis(cache_duration_ms),
        }
    }

    /// Get last command, using cache if valid
    pub fn get_last_command(&mut self, pane: &dyn mux::pane::Pane) -> Option<String> {
        // Check cache
        if let Some((cmd, timestamp)) = &self.last_command {
            if timestamp.elapsed() < self.cache_duration {
                return Some(cmd.clone());
            }
        }

        // Fetch new data
        let cmd = Self::fetch_last_command(pane)?;
        self.last_command = Some((cmd.clone(), Instant::now()));
        Some(cmd)
    }

    /// Get git branch, using cache if valid
    pub fn get_git_branch(&mut self, pane: &dyn mux::pane::Pane) -> Option<String> {
        // Check cache
        if let Some((branch, timestamp)) = &self.git_branch {
            if timestamp.elapsed() < self.cache_duration {
                return Some(branch.clone());
            }
        }

        // Fetch new data
        let branch = Self::fetch_git_branch(pane)?;
        self.git_branch = Some((branch.clone(), Instant::now()));
        Some(branch)
    }

    /// Fetch last command from pane
    fn fetch_last_command(pane: &dyn mux::pane::Pane) -> Option<String> {
        let proc_info = pane.get_foreground_process_info(CachePolicy::AllowStale)?;

        if proc_info.argv.is_empty() {
            return None;
        }

        let cmd = &proc_info.argv[0];
        let cmd_name = Path::new(cmd).file_name()?.to_str()?.to_string();

        // Filter shell commands
        if Self::is_shell_command(&cmd_name) {
            return None;
        }

        // Build command string with smart truncation
        let full_cmd = if proc_info.argv.len() > 1 {
            format!("{} {}", cmd_name, proc_info.argv[1..].join(" "))
        } else {
            cmd_name
        };

        Some(Self::truncate_command(&full_cmd, 20))
    }

    /// Fetch git branch from pane's current directory
    fn fetch_git_branch(pane: &dyn mux::pane::Pane) -> Option<String> {
        let cwd = pane.get_current_working_dir(CachePolicy::AllowStale)?;

        if cwd.scheme() != "file" {
            return None;
        }

        let path = cwd.to_file_path().ok()?;
        let git_dir = Self::find_git_dir(&path)?;

        let branch = Self::read_git_branch(&git_dir)?;
        let status = Self::get_git_status(&git_dir)?;

        Some(format!("git:{} {}", branch, status))
    }

    /// Check if command is a shell
    fn is_shell_command(cmd: &str) -> bool {
        let shells = ["bash", "zsh", "fish", "sh", "pwsh", "powershell", "cmd"];
        shells.iter().any(|&s| cmd.contains(s))
    }

    /// Truncate command with ellipsis
    fn truncate_command(cmd: &str, max_len: usize) -> String {
        if cmd.len() <= max_len {
            cmd.to_string()
        } else {
            format!("{}...", &cmd[..max_len.saturating_sub(3)])
        }
    }

    /// Find .git directory
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

    /// Read git branch from .git/HEAD
    fn read_git_branch(git_dir: &Path) -> Option<String> {
        let head_path = git_dir.join("HEAD");
        let head_content = std::fs::read_to_string(&head_path).ok()?;

        // Parse branch from ref: refs/heads/branch_name
        if head_content.starts_with("ref: refs/heads/") {
            Some(head_content[16..].trim().to_string())
        } else {
            // Detached HEAD
            Some("HEAD".to_string())
        }
    }

    /// Get git status indicator
    fn get_git_status(git_dir: &Path) -> Option<String> {
        // Simple check: if index has changes
        let index_path = git_dir.join("index");
        if !index_path.exists() {
            return Some("✓".to_string());
        }

        // For simplicity, just return clean indicator
        // A more complete implementation would parse git status
        Some("✓".to_string())
    }
}
