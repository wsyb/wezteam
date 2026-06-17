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

pub type Request = IpcRequest;
pub type Response = IpcResponse;

pub fn error_response(msg: &str) -> IpcResponse {
    IpcResponse {
        ok: false,
        error: Some(msg.to_string()),
        status: None,
        target: None,
        tab: None,
        lines: None,
        text: None,
        tabs: None,
    }
}
