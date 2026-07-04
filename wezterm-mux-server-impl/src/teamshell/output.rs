use super::protocol::IpcResponse;

pub fn format_response(response: &IpcResponse) -> String {
    if !response.ok {
        return format!(
            "Error: {}",
            response.error.as_deref().unwrap_or("unknown error")
        );
    }

    if let Some(states) = &response.states {
        return states
            .iter()
            .map(|s| {
                let mut parts = vec![format!("{} {}", s.tab_index, s.name)];
                if let Some(status) = &s.status {
                    parts.push(format!("[{}]", status));
                }
                if let Some(progress) = s.progress {
                    parts.push(format!("{}%", progress));
                }
                if let Some(task) = &s.task {
                    parts.push(format!("task={}", task));
                }
                if let Some(reason) = &s.blocked_reason {
                    parts.push(format!("blocked={}", reason));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");
    }

    if let Some(state) = &response.state {
        let mut parts = vec![format!("{} {}", state.tab_index, state.name)];
        if let Some(status) = &state.status {
            parts.push(format!("[{}]", status));
        }
        if let Some(progress) = state.progress {
            parts.push(format!("{}%", progress));
        }
        if let Some(task) = &state.task {
            parts.push(format!("task={}", task));
        }
        if let Some(reason) = &state.blocked_reason {
            parts.push(format!("blocked={}", reason));
        }
        return parts.join(" ");
    }

    match response.status.as_deref() {
        Some("delivered") => response
            .target
            .map(|target| format!("OK → {target}"))
            .unwrap_or_else(|| "OK".to_string()),
        Some("created") => match (response.tab, response.text.as_deref()) {
            (Some(tab), Some(name)) if !name.is_empty() => format!("Created tab {tab} ({name})"),
            (Some(tab), _) => format!("Created tab {tab}"),
            _ => "Created".to_string(),
        },
        Some("closed") => response
            .target
            .map(|target| format!("Closed tab {target}"))
            .unwrap_or_else(|| "Closed".to_string()),
        Some("renamed") => match (response.target, response.text.as_deref()) {
            (Some(target), Some(new_name)) => format!("Tab {target} renamed to \"{new_name}\""),
            (Some(target), _) => format!("Tab {target} renamed"),
            _ => "Renamed".to_string(),
        },
        Some("reported") => response
            .target
            .map(|target| format!("Reported tab {target}"))
            .unwrap_or_else(|| "Reported".to_string()),
        Some(status) => status.to_string(),
        None => response.text.clone().unwrap_or_else(|| "OK".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::protocol::{IpcResponse, TabState};
    use super::format_response;

    #[test]
    fn error_format() {
        let resp = IpcResponse::error("tab not found".into());
        assert_eq!(format_response(&resp), "Error: tab not found");
    }

    #[test]
    fn status_list_format() {
        let states = vec![
            TabState {
                tab_index: 1,
                name: "Alice".into(),
                process_alive: true,
                last_output_ago_secs: 3,
                status: Some("running".into()),
                progress: Some(80),
                task: None,
                blocked_reason: None,
                last_report: None,
            },
            TabState {
                tab_index: 2,
                name: "Bob".into(),
                process_alive: true,
                last_output_ago_secs: 10,
                status: None,
                progress: None,
                task: None,
                blocked_reason: None,
                last_report: None,
            },
        ];
        let resp = IpcResponse::status_list(states);
        assert_eq!(format_response(&resp), "1 Alice [running] 80%\n2 Bob");
    }

    #[test]
    fn delivered_format() {
        let resp = IpcResponse::delivered(2);
        assert_eq!(format_response(&resp), "OK → 2");
    }

    #[test]
    fn created_with_name() {
        let resp = IpcResponse::created(3, "Charlie".into());
        assert_eq!(format_response(&resp), "Created tab 3 (Charlie)");
    }

    #[test]
    fn created_without_name() {
        let resp = IpcResponse::created(3, "".into());
        assert_eq!(format_response(&resp), "Created tab 3");
    }

    #[test]
    fn closed_format() {
        let resp = IpcResponse::closed(3);
        assert_eq!(format_response(&resp), "Closed tab 3");
    }

    #[test]
    fn renamed_format() {
        let resp = IpcResponse::renamed(2, "Bob".into());
        assert_eq!(format_response(&resp), "Tab 2 renamed to \"Bob\"");
    }

    #[test]
    fn reported_format() {
        let resp = IpcResponse::reported(2);
        assert_eq!(format_response(&resp), "Reported tab 2");
    }

    #[test]
    fn text_format() {
        let resp = IpcResponse {
            ok: true,
            error: None,
            status: None,
            target: Some(1),
            tab: None,
            lines: None,
            text: Some("hello world".into()),
            states: None,
            state: None,
        };
        assert_eq!(format_response(&resp), "hello world");
    }
}
