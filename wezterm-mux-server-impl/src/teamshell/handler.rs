use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{LazyLock, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use mux::pane::Pane;
use mux::tab;
use mux::Mux;
use portable_pty::CommandBuilder;
use wezterm_term::TerminalSize;

use super::protocol::{IpcRequest, IpcResponse, TabState};

struct ActivityInfo {
    last_change: Instant,
    content_hash: u64,
}

#[derive(Debug, Clone)]
struct ReportedState {
    status: Option<String>,
    progress: Option<u8>,
    task: Option<String>,
    blocked_reason: Option<String>,
    last_report: Option<String>,
}

static ACTIVITY_TRACKER: LazyLock<Mutex<HashMap<usize, ActivityInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static STATE_STORE: LazyLock<Mutex<HashMap<usize, ReportedState>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn simple_hash(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

fn now_timestamp_secs() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

pub struct Handler;

impl Handler {
    pub fn handle(request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::Type {
                tab_index,
                message,
                from_tab_id,
            } => Self::send(tab_index, &format_message(&message, from_tab_id)),
            IpcRequest::TypeRaw { tab_index, data } => Self::send_raw(tab_index, &data),
            IpcRequest::View {
                tab_index,
                line_count,
            } => Self::see(tab_index, Some(line_count)),
            IpcRequest::Status => Self::status(),
            IpcRequest::Report {
                tab_index,
                key,
                value,
            } => Self::report(tab_index, &key, value.as_deref()),
            IpcRequest::Query { tab_index } => Self::query(tab_index),
            IpcRequest::Open {
                name,
                command,
                args,
                cwd,
                env,
            } => Self::open(&name, command, args, cwd, env),
            IpcRequest::Close { tab_index } => Self::close(tab_index),
            IpcRequest::Name {
                tab_index,
                new_name,
            } => Self::name(tab_index, &new_name),
        }
    }

    pub fn status() -> IpcResponse {
        let mux = Mux::get();
        let mut result = Vec::new();

        for window_id in mux.iter_windows() {
            if let Some(win) = mux.get_window(window_id) {
                for tab in win.iter() {
                    let tab_id = tab.tab_id();
                    let external_id = tab_id + 1;
                    let title = tab.get_title();
                    let name = if title.is_empty() {
                        format!("tab_{}", external_id)
                    } else {
                        title
                    };

                    let (process_alive, last_output_ago_secs) = {
                        if let Some(pane) = tab.get_active_pane() {
                            let dims = pane.get_dimensions();
                            let end = dims.physical_top + dims.viewport_rows as isize;
                            let start = end.saturating_sub(1);
                            let (_, lines) = pane.get_lines(start..end);
                            let last_line = lines
                                .last()
                                .map(|l| l.as_str().to_string())
                                .unwrap_or_default();

                            let mut tracker = ACTIVITY_TRACKER.lock().unwrap();
                            let now = Instant::now();
                            let entry = tracker.entry(external_id).or_insert(ActivityInfo {
                                last_change: now,
                                content_hash: 0,
                            });

                            let current_hash = simple_hash(&last_line);
                            if current_hash != entry.content_hash {
                                entry.content_hash = current_hash;
                                entry.last_change = now;
                            }

                            (true, now.duration_since(entry.last_change).as_secs())
                        } else {
                            (false, u64::MAX)
                        }
                    };

                    let reported = STATE_STORE
                        .lock()
                        .unwrap()
                        .get(&external_id)
                        .cloned();

                    result.push(TabState {
                        tab_index: external_id,
                        name,
                        process_alive,
                        last_output_ago_secs,
                        status: reported.as_ref().and_then(|r| r.status.clone()),
                        progress: reported.as_ref().and_then(|r| r.progress),
                        task: reported.as_ref().and_then(|r| r.task.clone()),
                        blocked_reason: reported.as_ref().and_then(|r| r.blocked_reason.clone()),
                        last_report: reported.as_ref().and_then(|r| r.last_report.clone()),
                    });
                }
            }
        }

        result.sort_by_key(|t| t.tab_index);
        IpcResponse::status_list(result)
    }

    pub fn report(tab_index: usize, key: &str, value: Option<&str>) -> IpcResponse {
        let mut store = STATE_STORE.lock().unwrap();
        let entry = store.entry(tab_index).or_insert(ReportedState {
            status: None,
            progress: None,
            task: None,
            blocked_reason: None,
            last_report: None,
        });

        let now = now_timestamp_secs();
        entry.last_report = Some(now);

        match key {
            "progress" => {
                if let Some(v) = value {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 100 => entry.progress = Some(n),
                        Ok(n) => {
                            return IpcResponse::error(format!(
                                "progress out of range: {n} (must be 0-100)"
                            ))
                        }
                        Err(_) => {
                            return IpcResponse::error(format!(
                                "invalid progress value: {v}"
                            ))
                        }
                    }
                }
            }
            "task" => {
                entry.task = value.map(|s| s.to_string());
            }
            "status" => {
                entry.status = value.map(|s| s.to_string());
            }
            "blocked" => {
                entry.status = Some("blocked".to_string());
                entry.blocked_reason = value.map(|s| s.to_string());
            }
            "done" => {
                entry.status = Some("done".to_string());
                entry.progress = Some(100);
            }
            _ => {
                return IpcResponse::error(format!("unknown report key: {key}"));
            }
        }

        IpcResponse::reported(tab_index)
    }

    pub fn query(tab_index: usize) -> IpcResponse {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(tab_index);

        let tab = match mux.get_tab(tab_id) {
            Some(t) => t,
            None => return IpcResponse::error(format!("tab {tab_index} not found")),
        };

        let title = tab.get_title();
        let name = if title.is_empty() {
            format!("tab_{tab_index}")
        } else {
            title
        };

        let (process_alive, last_output_ago_secs) = {
            if let Some(pane) = tab.get_active_pane() {
                let dims = pane.get_dimensions();
                let end = dims.physical_top + dims.viewport_rows as isize;
                let start = end.saturating_sub(1);
                let (_, lines) = pane.get_lines(start..end);
                let last_line = lines
                    .last()
                    .map(|l| l.as_str().to_string())
                    .unwrap_or_default();

                let mut tracker = ACTIVITY_TRACKER.lock().unwrap();
                let now = Instant::now();
                let entry = tracker.entry(tab_index).or_insert(ActivityInfo {
                    last_change: now,
                    content_hash: 0,
                });

                let current_hash = simple_hash(&last_line);
                if current_hash != entry.content_hash {
                    entry.content_hash = current_hash;
                    entry.last_change = now;
                }

                (true, now.duration_since(entry.last_change).as_secs())
            } else {
                (false, u64::MAX)
            }
        };

        let reported = STATE_STORE
            .lock()
            .unwrap()
            .get(&tab_index)
            .cloned();

        IpcResponse::queried(TabState {
            tab_index,
            name,
            process_alive,
            last_output_ago_secs,
            status: reported.as_ref().and_then(|r| r.status.clone()),
            progress: reported.as_ref().and_then(|r| r.progress),
            task: reported.as_ref().and_then(|r| r.task.clone()),
            blocked_reason: reported.as_ref().and_then(|r| r.blocked_reason.clone()),
            last_report: reported.as_ref().and_then(|r| r.last_report.clone()),
        })
    }

    pub fn send(target: usize, text: &str) -> IpcResponse {
        match Self::get_active_pane(target) {
            Some(pane) => match pane.send_paste(text) {
                Ok(()) => IpcResponse::delivered(target),
                Err(e) => IpcResponse::error(format!("send_paste failed: {e:#}")),
            },
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn send_raw(target: usize, data: &str) -> IpcResponse {
        match Self::get_active_pane(target) {
            Some(pane) => {
                let mut writer = pane.writer();
                if let Err(e) = writer.write_all(data.as_bytes()) {
                    return IpcResponse::error(format!("write to PTY failed: {e:#}"));
                }
                if let Err(e) = writer.flush() {
                    return IpcResponse::error(format!("flush PTY failed: {e:#}"));
                }
                IpcResponse::delivered(target)
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn see(target: usize, num_lines: Option<usize>) -> IpcResponse {
        match Self::get_active_pane(target) {
            Some(pane) => {
                let dims = pane.get_dimensions();
                let line_count = num_lines.unwrap_or(dims.viewport_rows) as isize;
                let end = dims.physical_top + dims.viewport_rows as isize;
                let start = (end - line_count).max(dims.scrollback_top);

                let (_first_row, lines) = pane.get_lines(start..end);
                let rendered_lines = lines
                    .iter()
                    .map(|line| line.as_str().to_string())
                    .collect::<Vec<_>>();
                let text = rendered_lines.join("\n");

                IpcResponse::text(target, rendered_lines, text)
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn open(
        name: &str,
        command: Option<String>,
        args: Option<Vec<String>>,
        cwd: Option<String>,
        env: Option<Vec<(String, String)>>,
    ) -> IpcResponse {
        let mux = Mux::get();
        let domain = mux.default_domain();
        let size = TerminalSize::default();
        let predicted_id = tab::next_tab_id() + 1;
        let cmd = command.map(|command| build_command(name, command, args, cwd, env, predicted_id));

        let windows = mux.iter_windows();
        let window_id = windows
            .first()
            .copied()
            .or_else(|| Some(*mux.new_empty_window(None, None)));

        match window_id {
            Some(wid) => match smol::block_on(domain.spawn(size, cmd, None, wid)) {
                Ok(tab) => {
                    if !name.is_empty() {
                        tab.set_title(name);
                    }
                    let external_id = tab.tab_id() + 1;
                    let now = Instant::now();
                    ACTIVITY_TRACKER.lock().unwrap().insert(
                        external_id,
                        ActivityInfo {
                            last_change: now,
                            content_hash: 0,
                        },
                    );
                    IpcResponse::created(external_id, name.to_string())
                }
                Err(e) => IpcResponse::error(format!("open failed: {e:#}")),
            },
            None => IpcResponse::error("open failed: no available window".to_string()),
        }
    }

    pub fn close(target: usize) -> IpcResponse {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(target);

        match mux.get_tab(tab_id) {
            Some(tab) => {
                if let Some(pane) = tab.get_active_pane() {
                    pane.kill();
                }
                mux.remove_tab(tab_id);
                ACTIVITY_TRACKER.lock().unwrap().remove(&target);
                STATE_STORE.lock().unwrap().remove(&target);
                IpcResponse::closed(target)
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn name(target: usize, new_name: &str) -> IpcResponse {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(target);

        match mux.get_tab(tab_id) {
            Some(tab) => {
                tab.set_title(new_name);
                IpcResponse::renamed(target, new_name.to_string())
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    fn get_active_pane(external_id: usize) -> Option<std::sync::Arc<dyn Pane>> {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(external_id);
        mux.get_tab(tab_id)?.get_active_pane()
    }

    fn id_to_tab_id(external_id: usize) -> usize {
        external_id.saturating_sub(1)
    }
}

fn get_tab_name(external_id: usize) -> String {
    let mux = Mux::get();
    let tab_id = external_id.saturating_sub(1);
    match mux.get_tab(tab_id) {
        Some(tab) => {
            let title = tab.get_title();
            if title.is_empty() {
                format!("tab_{external_id}")
            } else {
                title
            }
        }
        None => format!("tab_{external_id}"),
    }
}

fn format_message(message: &str, from_tab_id: Option<usize>) -> String {
    match from_tab_id {
        Some(from_tab_id) => {
            let name = get_tab_name(from_tab_id);
            format!("[TeamShell 消息] 来自 {name}:\n{message}\n")
        }
        None => format!("[TeamShell 消息] {message}\n"),
    }
}

fn build_command(
    name: &str,
    command: String,
    args: Option<Vec<String>>,
    cwd: Option<String>,
    env: Option<Vec<(String, String)>>,
    tab_id: usize,
) -> CommandBuilder {
    let mut builder = CommandBuilder::new(command);
    builder.env("TEAMSH_NAME", name);
    builder.env("TEAMSH_TAB_ID", tab_id.to_string());
    builder.env("TEAMSH_PLATFORM", std::env::consts::OS);

    if let Some(args) = args {
        builder.args(args);
    }
    if let Some(cwd) = cwd {
        builder.cwd(cwd);
    }
    if let Some(env_) = env {
        for (key, value) in env_ {
            builder.env(key, value);
        }
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_injects_env_vars() {
        let command = build_command("星河", "pwsh".to_string(), None, None, None, 3);

        assert_eq!(
            command.get_env("TEAMSH_NAME"),
            Some(std::ffi::OsStr::new("星河"))
        );
        assert_eq!(
            command.get_env("TEAMSH_TAB_ID"),
            Some(std::ffi::OsStr::new("3"))
        );
    }

    #[test]
    fn simple_hash_returns_consistent_value() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn simple_hash_differs_for_different_input() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn report_stores_state() {
        let resp = Handler::report(99, "status", Some("running"));
        assert!(resp.ok);
        assert_eq!(resp.target, Some(99));

        let store = STATE_STORE.lock().unwrap();
        let state = store.get(&99).unwrap();
        assert_eq!(state.status.as_deref(), Some("running"));
    }

    #[test]
    fn report_done_sets_progress_100() {
        let resp = Handler::report(98, "done", None);
        assert!(resp.ok);

        let store = STATE_STORE.lock().unwrap();
        let state = store.get(&98).unwrap();
        assert_eq!(state.status.as_deref(), Some("done"));
        assert_eq!(state.progress, Some(100));
    }

    #[test]
    fn report_blocked_sets_status_and_reason() {
        let resp = Handler::report(97, "blocked", Some("need permission"));
        assert!(resp.ok);

        let store = STATE_STORE.lock().unwrap();
        let state = store.get(&97).unwrap();
        assert_eq!(state.status.as_deref(), Some("blocked"));
        assert_eq!(state.blocked_reason.as_deref(), Some("need permission"));
    }

    #[test]
    fn report_unknown_key_returns_error() {
        let resp = Handler::report(96, "unknown_key", None);
        assert!(!resp.ok);
    }

    #[test]
    fn report_progress_out_of_range_returns_error() {
        let resp = Handler::report(95, "progress", Some("150"));
        assert!(!resp.ok);
        assert!(resp.error.unwrap().contains("out of range"));
    }

    #[test]
    fn report_progress_invalid_value_returns_error() {
        let resp = Handler::report(94, "progress", Some("abc"));
        assert!(!resp.ok);
        assert!(resp.error.unwrap().contains("invalid"));
    }

    #[test]
    fn report_progress_boundary_values() {
        let resp0 = Handler::report(93, "progress", Some("0"));
        assert!(resp0.ok);
        let resp100 = Handler::report(92, "progress", Some("100"));
        assert!(resp100.ok);
        let resp101 = Handler::report(91, "progress", Some("101"));
        assert!(!resp101.ok);
    }
}
