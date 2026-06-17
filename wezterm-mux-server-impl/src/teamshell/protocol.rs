use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum IpcRequest {
    #[serde(rename = "type")]
    Type {
        tab_index: usize,
        message: String,
        from_tab_id: Option<usize>,
    },
    #[serde(rename = "type_raw")]
    TypeRaw { tab_index: usize, data: String },
    #[serde(rename = "view")]
    View { tab_index: usize, line_count: usize },
    #[serde(rename = "list")]
    List,
    #[serde(rename = "open")]
    Open {
        name: String,
        command: Option<String>,
        args: Option<Vec<String>>,
        cwd: Option<String>,
        env: Option<Vec<(String, String)>>,
    },
    #[serde(rename = "close")]
    Close { tab_index: usize },
    #[serde(rename = "name")]
    Name { tab_index: usize, new_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<TabInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub index: usize,
    pub name: String,
}

impl IpcRequest {
    pub fn command_name(&self) -> &'static str {
        match self {
            Self::Type { .. } => "type",
            Self::TypeRaw { .. } => "type_raw",
            Self::View { .. } => "view",
            Self::List => "list",
            Self::Open { .. } => "open",
            Self::Close { .. } => "close",
            Self::Name { .. } => "name",
        }
    }
}

impl IpcResponse {
    pub fn error(error: String) -> Self {
        Self {
            ok: false,
            error: Some(error),
            status: None,
            target: None,
            tab: None,
            lines: None,
            text: None,
            tabs: None,
        }
    }

    pub fn list(tabs: Vec<TabInfo>) -> Self {
        Self {
            ok: true,
            error: None,
            status: None,
            target: None,
            tab: None,
            lines: None,
            text: None,
            tabs: Some(tabs),
        }
    }

    pub fn delivered(target: usize) -> Self {
        Self {
            ok: true,
            error: None,
            status: Some("delivered".to_string()),
            target: Some(target),
            tab: None,
            lines: None,
            text: None,
            tabs: None,
        }
    }

    pub fn text(target: usize, lines: Vec<String>, text: String) -> Self {
        Self {
            ok: true,
            error: None,
            status: None,
            target: Some(target),
            tab: None,
            lines: Some(lines),
            text: Some(text),
            tabs: None,
        }
    }

    pub fn created(tab: usize, name: String) -> Self {
        Self {
            ok: true,
            error: None,
            status: Some("created".to_string()),
            target: None,
            tab: Some(tab),
            lines: None,
            text: Some(name),
            tabs: None,
        }
    }

    pub fn closed(target: usize) -> Self {
        Self {
            ok: true,
            error: None,
            status: Some("closed".to_string()),
            target: Some(target),
            tab: None,
            lines: None,
            text: None,
            tabs: None,
        }
    }

    pub fn renamed(target: usize, new_name: String) -> Self {
        Self {
            ok: true,
            error: None,
            status: Some("renamed".to_string()),
            target: Some(target),
            tab: None,
            lines: None,
            text: Some(new_name),
            tabs: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_response_sets_error_fields() {
        let response = IpcResponse::error("boom".to_string());

        assert!(!response.ok);
        assert_eq!(response.error.as_deref(), Some("boom"));
    }

    #[test]
    fn list_response_sets_tabs() {
        let response = IpcResponse::list(vec![TabInfo {
            index: 6,
            name: "星河".to_string(),
        }]);

        assert!(response.ok);
        let tabs = response.tabs.as_ref().unwrap();
        assert_eq!(tabs[0].index, 6);
        assert_eq!(tabs[0].name, "星河");
    }

    #[test]
    fn delivered_response_sets_status_and_target() {
        let response = IpcResponse::delivered(2);

        assert!(response.ok);
        assert_eq!(response.status.as_deref(), Some("delivered"));
        assert_eq!(response.target, Some(2));
    }

    #[test]
    fn text_response_sets_lines_and_text() {
        let response = IpcResponse::text(2, vec!["a".to_string()], "a".to_string());

        assert!(response.ok);
        assert_eq!(response.target, Some(2));
        assert_eq!(response.lines.as_ref().unwrap(), &vec!["a".to_string()]);
        assert_eq!(response.text.as_deref(), Some("a"));
    }

    #[test]
    fn created_response_sets_status_tab_and_name() {
        let response = IpcResponse::created(3, "Mimo".to_string());

        assert!(response.ok);
        assert_eq!(response.status.as_deref(), Some("created"));
        assert_eq!(response.tab, Some(3));
        assert_eq!(response.text.as_deref(), Some("Mimo"));
    }

    #[test]
    fn closed_response_sets_status_and_target() {
        let response = IpcResponse::closed(3);

        assert!(response.ok);
        assert_eq!(response.status.as_deref(), Some("closed"));
        assert_eq!(response.target, Some(3));
    }

    #[test]
    fn renamed_response_sets_status_target_and_name() {
        let response = IpcResponse::renamed(3, "云雀".to_string());

        assert!(response.ok);
        assert_eq!(response.status.as_deref(), Some("renamed"));
        assert_eq!(response.target, Some(3));
        assert_eq!(response.text.as_deref(), Some("云雀"));
    }

    #[test]
    fn command_name_returns_names_for_all_request_variants() {
        let cases = vec![
            (
                IpcRequest::Type {
                    tab_index: 1,
                    message: String::new(),
                    from_tab_id: None,
                },
                "type",
            ),
            (
                IpcRequest::TypeRaw {
                    tab_index: 1,
                    data: String::new(),
                },
                "type_raw",
            ),
            (
                IpcRequest::View {
                    tab_index: 1,
                    line_count: 10,
                },
                "view",
            ),
            (IpcRequest::List, "list"),
            (
                IpcRequest::Open {
                    name: String::new(),
                    command: None,
                    args: None,
                    cwd: None,
                    env: None,
                },
                "open",
            ),
            (IpcRequest::Close { tab_index: 1 }, "close"),
            (
                IpcRequest::Name {
                    tab_index: 1,
                    new_name: String::new(),
                },
                "name",
            ),
        ];

        for (request, expected) in cases {
            assert_eq!(request.command_name(), expected);
        }
    }
}
