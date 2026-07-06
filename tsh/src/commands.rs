use crate::ipc;
use crate::protocol::{Request, Response};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Output {
    Stdout(String),
    Stderr(String),
}

fn format_response(response: &Response) -> Output {
    if !response.ok {
        return Output::Stderr(format!(
            "Error: {}\n",
            response.error.as_deref().unwrap_or("unknown error")
        ));
    }

    if let Some(states) = &response.states {
        let mut text = String::new();
        for s in states {
            let activity_icon = if !s.process_alive {
                "⚫"
            } else if s.last_output_ago_secs > 120 {
                "🔴"
            } else if s.last_output_ago_secs > 10 {
                "🟡"
            } else {
                "🟢"
            };

            let activity_label = if !s.process_alive {
                "离线".to_string()
            } else if s.last_output_ago_secs > 120 {
                "静默".to_string()
            } else if s.last_output_ago_secs > 10 {
                "缓慢".to_string()
            } else {
                "活跃".to_string()
            };

            let ago = if !s.process_alive {
                "进程已退出".to_string()
            } else if s.last_output_ago_secs == 0 {
                "刚刚".to_string()
            } else if s.last_output_ago_secs < 60 {
                format!("{}秒前", s.last_output_ago_secs)
            } else {
                format!("{}分钟前", s.last_output_ago_secs / 60)
            };

            text.push_str(&format!(
                "{}号({}) {} {} 最后活动: {}",
                s.tab_index, s.name, activity_icon, activity_label, ago
            ));

            if let Some(task) = &s.task {
                text.push_str(&format!(" 任务: {}", task));
            }
            if let Some(progress) = s.progress {
                text.push_str(&format!(" 进度: {}%", progress));
            }
            if let Some(status) = &s.status {
                match status.as_str() {
                    "blocked" => {
                        if let Some(reason) = &s.blocked_reason {
                            text.push_str(&format!(" ⚠️阻塞: {}", reason));
                        } else {
                            text.push_str(" ⚠️阻塞");
                        }
                    }
                    "done" => text.push_str(" ✅完成"),
                    _ => {}
                }
            }

            text.push('\n');
        }
        return Output::Stdout(text);
    }

    if let Some(state) = &response.state {
        let mut text = String::new();

        let status_str = match state.status.as_deref() {
            Some("running") => "运行中",
            Some("idle") => "空闲",
            Some("blocked") => "阻塞",
            Some("done") => "完成",
            Some("error") => "错误",
            other => other.unwrap_or("未知"),
        };

        text.push_str(&format!(
            "{}号({}) 状态: {}",
            state.tab_index, state.name, status_str
        ));

        if let Some(progress) = state.progress {
            text.push_str(&format!(" | 进度: {}%", progress));
        }
        if let Some(task) = &state.task {
            text.push_str(&format!(" | 任务: {}", task));
        }
        if let Some(reason) = &state.blocked_reason {
            text.push_str(&format!(" | 原因: {}", reason));
        }

        let ago = if state.last_output_ago_secs == 0 {
            "刚刚".to_string()
        } else if state.last_output_ago_secs < 60 {
            format!("{}秒前", state.last_output_ago_secs)
        } else {
            format!("{}分钟前", state.last_output_ago_secs / 60)
        };
        text.push_str(&format!(" | 最后活动: {}", ago));

        text.push('\n');
        return Output::Stdout(text);
    }

    let text = match response.status.as_deref() {
        Some("delivered") => match response.target {
            Some(target) => format!("OK → {target}\n"),
            None => "OK\n".to_string(),
        },
        Some("created") => match (response.tab, response.text.as_deref()) {
            (Some(tab), Some(name)) if !name.is_empty() => format!("Created tab {tab} ({name})\n"),
            (Some(tab), _) => format!("Created tab {tab}\n"),
            _ => "Created\n".to_string(),
        },
        Some("closed") => match response.target {
            Some(target) => format!("Closed tab {target}\n"),
            None => "Closed\n".to_string(),
        },
        Some("renamed") => match (response.target, response.text.as_deref()) {
            (Some(target), Some(new_name)) => format!("Tab {target} renamed to \"{new_name}\"\n"),
            (Some(target), _) => format!("Tab {target} renamed\n"),
            _ => "Renamed\n".to_string(),
        },
        Some("reported") => "✅ 已记录\n".to_string(),
        Some(status) => format!("{status}\n"),
        None => response.text.clone().unwrap_or_else(|| "OK\n".to_string()),
    };

    Output::Stdout(text)
}

/// 输出纯文本到 stdout/stderr 并设置进程退出码
fn output(response: &Response) {
    match format_response(response) {
        Output::Stdout(text) => print!("{text}"),
        Output::Stderr(text) => {
            eprint!("{text}");
            std::process::exit(1);
        }
    }
}

/// Read the sender tab ID from the TEAMSH_TAB_ID environment variable.
/// Returns None if not set (e.g. when running from external CLI).
fn sender_tab_id() -> Option<usize> {
    std::env::var("TEAMSH_TAB_ID")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
}

// ============================================================
// type — 在指定工位打字
// ============================================================

/// tsh type <id> "文本"         → 在指定工位敲入文本并回车
/// tsh type <id> --no-enter "文本" → 敲入文本，不回车
/// tsh type <id> --key "\x03"       → 发送按键
pub fn cmd_type(id: usize, message: Option<&str>, auto_enter: bool, key: Option<&str>) {
    if let Some(key_str) = key {
        let expanded = expand_escapes(key_str);
        let request = Request::TypeRaw {
            tab_index: id,
            data: expanded,
        };
        match ipc::send_request(&request) {
            Ok(resp) => output(&resp),
            Err(e) => output(&crate::protocol::error_response(&e)),
        }
    } else if let Some(msg) = message {
        let request = Request::Type {
            tab_index: id,
            message: msg.to_string(),
            from_tab_id: sender_tab_id(),
        };
        match ipc::send_request(&request) {
            Ok(resp) => {
                output(&resp);
                if auto_enter {
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    let raw_request = Request::TypeRaw {
                        tab_index: id,
                        data: "\r".to_string(),
                    };
                    let _ = ipc::send_request(&raw_request);
                }
            }
            Err(e) => output(&crate::protocol::error_response(&e)),
        }
    } else {
        eprintln!("错误：请提供文本内容或使用 --key 发送按键");
        std::process::exit(1);
    }
}

// ============================================================
// view — 读取工位屏幕
// ============================================================

/// tsh view <id> [lines]  → 读取并清理输出
pub fn cmd_view(id: usize, lines: usize) {
    let request = Request::View {
        tab_index: id,
        line_count: lines,
    };
    match ipc::send_request(&request) {
        Ok(mut resp) => {
            if let Some(ref mut lines) = resp.lines {
                for line in lines.iter_mut() {
                    *line = strip_ansi_codes(line);
                    *line = strip_control_chars(line);
                }
                for line in lines.iter_mut() {
                    *line = line.trim_end().to_string();
                }
                collapse_blank_lines(lines);
                while lines.first().is_some_and(|l| l.is_empty()) {
                    lines.remove(0);
                }
                while lines.last().is_some_and(|l| l.is_empty()) {
                    lines.pop();
                }
                resp.text = Some(lines.join("\n"));
                resp.lines = None;
            }
            output(&resp)
        }
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// status — 查看团队状态看板
// ============================================================

pub fn cmd_status() {
    let request = Request::Status;
    match ipc::send_request(&request) {
        Ok(resp) => output(&resp),
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// report — 报告当前工位状态
// ============================================================

pub fn cmd_report(key: &str, value: Option<&str>) {
    let tab_index = match sender_tab_id() {
        Some(id) => id,
        None => {
            eprintln!("错误：无法获取当前工位编号（TEAMSH_TAB_ID 未设置）");
            std::process::exit(1);
        }
    };

    let request = Request::Report {
        tab_index,
        key: key.to_string(),
        value: value.map(|s| s.to_string()),
    };
    match ipc::send_request(&request) {
        Ok(resp) => output(&resp),
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// query — 查询指定工位状态
// ============================================================

pub fn cmd_query(id: usize) {
    let request = Request::Query { tab_index: id };
    match ipc::send_request(&request) {
        Ok(resp) => output(&resp),
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// open — 招新成员
// ============================================================

/// tsh open "name" command arg1 arg2     → 执行命令
/// tsh open "name" command --auto-shell   → 用最佳 shell 包装
/// tsh open "name" command --init-prompt "msg" → 创建后自动发入职消息
pub fn cmd_open(
    name: &str,
    command: Option<&str>,
    args: &[&str],
    cwd: Option<&str>,
    env: &[(String, String)],
    auto_shell: bool,
    init_prompt: Option<&str>,
) {
    let final_command;
    let final_args;

    if auto_shell {
        if let Some(cmd) = command {
            let shell = detect_shell();
            let full_command = format!("{} {}", cmd, args.join(" "));
            let (shell_cmd, shell_args) = wrap_with_shell(&shell, &full_command);
            final_command = Some(shell_cmd);
            final_args = Some(shell_args);
        } else {
            final_command = None;
            final_args = None;
        }
    } else {
        final_command = command.map(|s| s.to_string());
        final_args = if args.is_empty() {
            None
        } else {
            Some(args.iter().map(|s| s.to_string()).collect())
        };
    }

    let request = Request::Open {
        name: name.to_string(),
        command: final_command,
        args: final_args,
        cwd: cwd.map(|s| s.to_string()),
        env: if env.is_empty() {
            None
        } else {
            Some(env.to_vec())
        },
    };
    match ipc::send_request(&request) {
        Ok(resp) => {
            if let Some(prompt) = init_prompt {
                if resp.ok {
                    if let Some(tab_id) = resp.tab {
                        std::thread::sleep(std::time::Duration::from_millis(3000));
                        let send_req = Request::Type {
                            tab_index: tab_id,
                            message: prompt.to_string(),
                            from_tab_id: Some(0),
                        };
                        let _ = ipc::send_request(&send_req);
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let enter_req = Request::TypeRaw {
                            tab_index: tab_id,
                            data: "\r".to_string(),
                        };
                        let _ = ipc::send_request(&enter_req);
                    }
                }
            }
            output(&resp)
        }
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// close — 让成员离开
// ============================================================

/// tsh close <id>
pub fn cmd_close(id: usize) {
    let request = Request::Close { tab_index: id };
    match ipc::send_request(&request) {
        Ok(resp) => output(&resp),
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// name — 给成员改名
// ============================================================

/// tsh name <id> "new_name"
pub fn cmd_name(id: usize, new_name: &str) {
    let request = Request::Name {
        tab_index: id,
        new_name: new_name.to_string(),
    };
    match ipc::send_request(&request) {
        Ok(resp) => output(&resp),
        Err(e) => output(&crate::protocol::error_response(&e)),
    }
}

// ============================================================
// 工具函数
// ============================================================

fn expand_escapes(s: &str) -> String {
    let mut result = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            match bytes[i + 1] {
                b'r' => {
                    result.push(b'\r');
                    i += 2;
                }
                b'n' => {
                    result.push(b'\n');
                    i += 2;
                }
                b't' => {
                    result.push(b'\t');
                    i += 2;
                }
                b'0' => {
                    result.push(b'\0');
                    i += 2;
                }
                b'x' if i + 3 < bytes.len() => {
                    let hi = bytes[i + 2];
                    let lo = bytes[i + 3];
                    if let (Some(h), Some(l)) = (hex_val(hi), hex_val(lo)) {
                        result.push(h << 4 | l);
                        i += 4;
                    } else {
                        result.push(bytes[i]);
                        i += 1;
                    }
                }
                b'\\' => {
                    result.push(b'\\');
                    i += 2;
                }
                _ => {
                    result.push(bytes[i]);
                    i += 1;
                }
            }
        } else {
            result.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(result).unwrap_or_default()
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphabetic() || next == '~' {
                        chars.next();
                        break;
                    }
                    chars.next();
                }
            } else if chars.peek() == Some(&']') {
                chars.next();
                for next in chars.by_ref() {
                    if next == '\x07' || next == '\x1b' {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn strip_control_chars(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c as u32 {
            0x0D => {}
            0x08 => {
                result.pop();
            }
            0x00..=0x06 | 0x0E..=0x1F => {}
            0x7F => {}
            _ => result.push(c),
        }
    }
    result
}

fn collapse_blank_lines(lines: &mut Vec<String>) {
    let mut i = 1;
    while i < lines.len() {
        if lines[i].is_empty() && lines[i - 1].is_empty() {
            lines.remove(i);
        } else {
            i += 1;
        }
    }
}

fn detect_shell() -> String {
    if cfg!(target_os = "windows") {
        for shell in &["pwsh", "cmd.exe"] {
            if *shell == "cmd.exe" {
                return shell.to_string();
            }
            if let Ok(output) = std::process::Command::new("cmd")
                .args(["/c", "where", shell])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if !stdout.trim().is_empty() {
                        return shell.to_string();
                    }
                }
            }
        }
        "powershell.exe".to_string()
    } else {
        for shell in &["fish", "zsh", "bash"] {
            if let Ok(output) = std::process::Command::new("which").arg(shell).output() {
                if output.status.success() {
                    return shell.to_string();
                }
            }
        }
        "sh".to_string()
    }
}

fn wrap_with_shell(shell: &str, command: &str) -> (String, Vec<String>) {
    if cfg!(target_os = "windows") {
        if shell == "pwsh" {
            (
                shell.to_string(),
                vec!["-Command".to_string(), command.to_string()],
            )
        } else {
            let parts: Vec<String> = command.split_whitespace().map(|s| s.to_string()).collect();
            let mut args = vec!["/c".to_string()];
            args.extend(parts);
            (shell.to_string(), args)
        }
    } else {
        (
            shell.to_string(),
            vec!["-c".to_string(), command.to_string()],
        )
    }
}

// ============================================================
// init — 初始化项目协议
// ============================================================

const PROTOCOL_VERSION: &str = env!("CARGO_PKG_VERSION");
const SEED_PROTOCOL: &str = include_str!("../../TeamShellProtocol.md");

fn protocol_start_marker() -> String {
    format!("<!-- TeamShell Protocol Start v{} -->", PROTOCOL_VERSION)
}
const PROTOCOL_END_MARKER: &str = "<!-- TeamShell Protocol End -->";
const PROTOCOL_START_PREFIX: &str = "<!-- TeamShell Protocol Start";
const KNOWN_AGENTS: &[&str] = &["claude", "codex", "gemini"];

fn agent_config_paths(agent: &str) -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;
    match agent {
        "claude" => vec![PathBuf::from("CLAUDE.md")],
        "codex" => vec![PathBuf::from(".codex").join("instructions.md")],
        "gemini" => vec![PathBuf::from(".gemini").join("GEMINI.md")],
        _ => vec![],
    }
}

fn discover_agents() -> Vec<String> {
    let mut found = Vec::new();
    for agent in KNOWN_AGENTS {
        let result = if cfg!(target_os = "windows") {
            std::process::Command::new("where")
                .arg(agent)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
        } else {
            std::process::Command::new("which")
                .arg(agent)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
        };
        if let Ok(status) = result {
            if status.success() {
                found.push(agent.to_string());
            }
        }
    }
    found
}

fn build_protocol_block() -> String {
    format!(
        "{}\n{}\n{}",
        protocol_start_marker(),
        SEED_PROTOCOL,
        PROTOCOL_END_MARKER
    )
}

enum WriteAction {
    Created,
    Appended,
    Replaced,
    Skipped,
    DryRun,
}

fn write_protocol_to_file(path: &std::path::Path, force: bool, dry_run: bool) -> WriteAction {
    use std::fs;

    let block = build_protocol_block();

    if !path.exists() {
        if !dry_run {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(path, &block).unwrap_or_else(|_| panic!("无法写入 {}", path.display()));
        }
        return WriteAction::Created;
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let start_pos = content.find(PROTOCOL_START_PREFIX);
    let end_pos = content.find(PROTOCOL_END_MARKER);

    match (start_pos, end_pos) {
        (Some(s), Some(e)) => {
            let start_line_end = content[s..].find('\n').unwrap_or(content.len() - s);
            let existing_start = &content[s..s + start_line_end];
            let is_same_version = existing_start.contains(PROTOCOL_VERSION);

            if is_same_version && !force {
                return WriteAction::Skipped;
            }

            if dry_run {
                return if is_same_version {
                    WriteAction::Skipped
                } else {
                    WriteAction::DryRun
                };
            }

            let end_with_marker = e + PROTOCOL_END_MARKER.len();
            let before = &content[..s];
            let after = content[end_with_marker..].trim_start_matches('\n');
            let new_content = format!("{}{}{}", before.trim_end(), "\n\n", block);
            let new_content = if !after.trim().is_empty() {
                format!("{}\n\n{}", new_content, after.trim_end())
            } else {
                format!("{}\n", new_content)
            };
            fs::write(path, &new_content).unwrap_or_else(|_| panic!("无法写入 {}", path.display()));
            WriteAction::Replaced
        }
        _ => {
            if dry_run {
                return WriteAction::DryRun;
            }
            let new_content = format!("{}\n\n{}\n", content.trim_end(), block);
            fs::write(path, &new_content).unwrap_or_else(|_| panic!("无法写入 {}", path.display()));
            WriteAction::Appended
        }
    }
}

fn has_protocol(path: &std::path::Path) -> bool {
    if !path.exists() {
        return false;
    }
    let content = std::fs::read_to_string(path).unwrap_or_default();
    content.contains(PROTOCOL_START_PREFIX)
}

fn confirm_overwrite(file_name: &str) -> bool {
    use std::io::{self, Write};
    eprint!("{} 已包含 TeamShell 协议，是否覆盖？[y/N] ", file_name);
    io::stderr().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    input.trim().eq_ignore_ascii_case("y")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TabState;

    fn ok_response() -> Response {
        Response {
            ok: true,
            error: None,
            status: None,
            target: None,
            tab: None,
            lines: None,
            text: None,
            states: None,
            state: None,
        }
    }

    #[test]
    fn format_error_to_stderr() {
        let response = Response {
            ok: false,
            error: Some("tab 99 not found".to_string()),
            ..ok_response()
        };

        assert_eq!(
            format_response(&response),
            Output::Stderr("Error: tab 99 not found\n".to_string())
        );
    }

    #[test]
    fn format_status_as_lines() {
        let response = Response {
            states: Some(vec![
                TabState {
                    tab_index: 1,
                    name: "Alice".to_string(),
                    process_alive: true,
                    last_output_ago_secs: 5,
                    status: None,
                    progress: None,
                    task: None,
                    blocked_reason: None,
                    last_report: None,
                },
                TabState {
                    tab_index: 2,
                    name: "Bob".to_string(),
                    process_alive: false,
                    last_output_ago_secs: 300,
                    status: Some("done".to_string()),
                    progress: Some(100),
                    task: Some("重构模块".to_string()),
                    blocked_reason: None,
                    last_report: None,
                },
            ]),
            ..ok_response()
        };

        let result = format_response(&response);
        if let Output::Stdout(text) = result {
            assert!(text.contains("1号(Alice) 🟢 活跃"));
            assert!(text.contains("2号(Bob) ⚫ 离线"));
            assert!(text.contains("✅完成"));
        } else {
            panic!("Expected Stdout");
        }
    }

    #[test]
    fn format_query_detail() {
        let response = Response {
            state: Some(TabState {
                tab_index: 1,
                name: "Alice".to_string(),
                process_alive: true,
                last_output_ago_secs: 30,
                status: Some("running".to_string()),
                progress: Some(50),
                task: Some("写测试".to_string()),
                blocked_reason: None,
                last_report: None,
            }),
            ..ok_response()
        };

        let result = format_response(&response);
        if let Output::Stdout(text) = result {
            assert!(text.contains("1号(Alice) 状态: 运行中"));
            assert!(text.contains("进度: 50%"));
            assert!(text.contains("任务: 写测试"));
            assert!(text.contains("30秒前"));
        } else {
            panic!("Expected Stdout");
        }
    }

    #[test]
    fn format_delivered_status() {
        let response = Response {
            status: Some("delivered".to_string()),
            target: Some(2),
            ..ok_response()
        };

        assert_eq!(
            format_response(&response),
            Output::Stdout("OK → 2\n".to_string())
        );
    }

    #[test]
    fn format_created_status_with_name() {
        let response = Response {
            status: Some("created".to_string()),
            tab: Some(3),
            text: Some("Charlie".to_string()),
            ..ok_response()
        };

        assert_eq!(
            format_response(&response),
            Output::Stdout("Created tab 3 (Charlie)\n".to_string())
        );
    }

    #[test]
    fn format_closed_status() {
        let response = Response {
            status: Some("closed".to_string()),
            target: Some(3),
            ..ok_response()
        };

        assert_eq!(
            format_response(&response),
            Output::Stdout("Closed tab 3\n".to_string())
        );
    }

    #[test]
    fn format_renamed_status() {
        let response = Response {
            status: Some("renamed".to_string()),
            target: Some(2),
            text: Some("Bob".to_string()),
            ..ok_response()
        };

        assert_eq!(
            format_response(&response),
            Output::Stdout("Tab 2 renamed to \"Bob\"\n".to_string())
        );
    }

    #[test]
    fn format_text_without_extra_newline() {
        let response = Response {
            text: Some("hello\nworld\n".to_string()),
            ..ok_response()
        };

        assert_eq!(
            format_response(&response),
            Output::Stdout("hello\nworld\n".to_string())
        );
    }

    #[test]
    fn format_default_ok() {
        assert_eq!(
            format_response(&ok_response()),
            Output::Stdout("OK\n".to_string())
        );
    }

    // expand_escapes tests — literal text and \r \n \t \0 \xNN escapes

    #[test]
    fn escape_backslash_r() {
        assert_eq!(expand_escapes("\\r"), "\r");
    }

    #[test]
    fn escape_backslash_x() {
        assert_eq!(expand_escapes("\\x03"), "\x03");
        assert_eq!(expand_escapes("\\x1b"), "\x1b");
        assert_eq!(expand_escapes("\\x41"), "A");
    }

    #[test]
    fn escape_backslash_n() {
        assert_eq!(expand_escapes("\\n"), "\n");
    }

    #[test]
    fn escape_backslash_t() {
        assert_eq!(expand_escapes("\\t"), "\t");
    }

    #[test]
    fn escape_backslash_0() {
        assert_eq!(expand_escapes("\\0"), "\0");
    }

    #[test]
    fn escape_backslash_backslash() {
        assert_eq!(expand_escapes("\\\\"), "\\");
    }

    #[test]
    fn literal_text_passes_through() {
        assert_eq!(expand_escapes("hello world"), "hello world");
    }
}

pub fn cmd_init(force: bool, dry_run: bool, show: bool, include_files: &[String]) {
    use std::path::Path;

    if show {
        print!("{}", build_protocol_block());
        return;
    }

    let target_dir = std::env::current_dir().expect("无法获取当前工作目录");

    if !target_dir.is_dir() {
        eprintln!("错误：目录不存在: {}", target_dir.display());
        std::process::exit(1);
    }

    if let Some(home) = dirs_next::home_dir() {
        let target_canon = target_dir.canonicalize().unwrap_or(target_dir.clone());
        let home_canon = home.canonicalize().unwrap_or(home.clone());
        let target_str = target_canon.to_string_lossy().replace("\\\\?\\", "");
        let home_str = home_canon.to_string_lossy().replace("\\\\?\\", "");
        if target_str == home_str {
            eprintln!("错误：不能在用户主目录下运行 tsh init。");
            eprintln!("这会将协议写入全局配置文件（如 ~/CLAUDE.md），影响所有项目。");
            eprintln!("请 cd 到具体的项目目录后再运行。");
            std::process::exit(1);
        }
    }

    let explicit_files = !include_files.is_empty();

    let valid_files: Vec<&String> = include_files
        .iter()
        .filter(|f| {
            if !f.ends_with(".md") {
                eprintln!("警告：跳过 {}（非 .md 文件，请用 Ctrl+Alt+I 手动注入）", f);
                false
            } else {
                true
            }
        })
        .collect();

    let mut all_paths: Vec<std::path::PathBuf> = Vec::new();

    if explicit_files {
        for f in &valid_files {
            all_paths.push(Path::new(f.as_str()).to_path_buf());
        }
    } else {
        let agents = discover_agents();
        if agents.is_empty() {
            eprintln!(
                "错误：未在系统中找到任何已安装的 AI Agent（检测了 {}）。",
                KNOWN_AGENTS.join("、")
            );
            eprintln!("请先安装至少一个 Agent，或手动指定配置文件：tsh init ATOMCODE.md");
            std::process::exit(1);
        }
        println!("检测到已安装的 Agent: {}", agents.join(", "));

        for agent in &agents {
            for rel_path in agent_config_paths(agent) {
                all_paths.push(rel_path);
            }
        }
    }

    let mut created = Vec::<String>::new();
    let mut updated = Vec::<String>::new();
    let mut skipped = Vec::<String>::new();

    for rel_path in &all_paths {
        let abs_path = target_dir.join(rel_path);
        let display_name = rel_path
            .to_string_lossy()
            .replace('/', std::path::MAIN_SEPARATOR_STR.as_ref());

        if !force && !dry_run && has_protocol(&abs_path) && !confirm_overwrite(&display_name) {
            skipped.push(format!("{} (用户跳过)", display_name));
            continue;
        }

        match write_protocol_to_file(&abs_path, force, dry_run) {
            WriteAction::Created => created.push(display_name),
            WriteAction::Appended => updated.push(format!("{} (追加)", display_name)),
            WriteAction::Replaced => updated.push(format!("{} (覆盖)", display_name)),
            WriteAction::Skipped => skipped.push(format!("{} (已是最新)", display_name)),
            WriteAction::DryRun => {
                if has_protocol(&abs_path) {
                    println!("  ~ {} (将覆盖)", display_name);
                } else {
                    println!("  + {} (将创建)", display_name);
                }
            }
        }
    }

    if dry_run {
        println!("预览模式（未写入任何文件）：");
    } else {
        println!("TeamShell 协议同步完成：");
    }
    for file in &created {
        println!("  + {} (新建)", file);
    }
    for file in &updated {
        println!("  ✓ {}", file);
    }
    for file in &skipped {
        println!("  - {} (跳过)", file);
    }
    println!();
    println!("协议版本: v{}", PROTOCOL_VERSION);
}
