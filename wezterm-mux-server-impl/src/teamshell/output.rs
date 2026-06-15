use super::protocol::IpcResponse;

pub fn format_response(response: &IpcResponse) -> String {
    if !response.ok {
        return format!(
            "Error: {}",
            response.error.as_deref().unwrap_or("unknown error")
        );
    }

    if let Some(tabs) = &response.tabs {
        return tabs
            .iter()
            .map(|tab| format!("{} {}", tab.index, tab.name))
            .collect::<Vec<_>>()
            .join("\n");
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
        Some(status) => status.to_string(),
        None => response.text.clone().unwrap_or_else(|| "OK".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::super::protocol::{IpcResponse, TabInfo};
    use super::format_response;

    #[test]
    fn error_format() {
        let resp = IpcResponse::error("tab not found".into());
        assert_eq!(format_response(&resp), "Error: tab not found");
    }

    #[test]
    fn list_format() {
        let tabs = vec![
            TabInfo {
                index: 1,
                name: "Alice".into(),
            },
            TabInfo {
                index: 2,
                name: "Bob".into(),
            },
        ];
        let resp = IpcResponse::list(tabs);
        assert_eq!(format_response(&resp), "1 Alice\n2 Bob");
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
    fn text_format() {
        let resp = IpcResponse {
            ok: true,
            error: None,
            status: None,
            target: Some(1),
            tab: None,
            lines: None,
            text: Some("hello world".into()),
            tabs: None,
        };
        assert_eq!(format_response(&resp), "hello world");
    }
}
