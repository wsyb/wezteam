use wezterm_mux_server_impl::teamshell::protocol::{IpcRequest, IpcResponse, TabInfo};

#[test]
fn ipc_request_uses_cmd_tag_and_snake_case_names() {
    let request = IpcRequest::Send {
        tab_index: 2,
        message: "hello".to_string(),
        from_tab_id: Some(6),
    };

    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(value["cmd"], "send");
    assert_eq!(value["tab_index"], 2);
    assert_eq!(value["message"], "hello");
    assert_eq!(value["from_tab_id"], 6);
}

#[test]
fn command_name_does_not_include_request_payload() {
    let request = IpcRequest::SendRaw {
        tab_index: 2,
        data: "secret-token".to_string(),
    };

    assert_eq!(request.command_name(), "send_raw");
}

#[test]
fn ipc_response_omits_empty_optional_fields() {
    let response = IpcResponse {
        ok: true,
        error: None,
        status: Some("received".to_string()),
        target: None,
        tab: None,
        lines: None,
        text: None,
        tabs: Some(vec![TabInfo {
            index: 6,
            name: "星河".to_string(),
        }]),
    };

    let value = serde_json::to_value(&response).unwrap();

    assert_eq!(value["ok"], true);
    assert_eq!(value["status"], "received");
    assert!(value.get("error").is_none());
    assert!(value.get("target").is_none());
    assert_eq!(value["tabs"][0]["index"], 6);
    assert_eq!(value["tabs"][0]["name"], "星河");
}
