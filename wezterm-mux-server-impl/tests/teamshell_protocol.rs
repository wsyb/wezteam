use wezterm_mux_server_impl::teamshell::protocol::{IpcRequest, IpcResponse, TabState};

#[test]
fn ipc_request_uses_cmd_tag_and_snake_case_names() {
    let request = IpcRequest::Type {
        tab_index: 2,
        message: "hello".to_string(),
        from_tab_id: Some(6),
    };

    let value = serde_json::to_value(&request).unwrap();

    assert_eq!(value["cmd"], "type");
    assert_eq!(value["tab_index"], 2);
    assert_eq!(value["message"], "hello");
    assert_eq!(value["from_tab_id"], 6);
}

#[test]
fn command_name_does_not_include_request_payload() {
    let request = IpcRequest::TypeRaw {
        tab_index: 2,
        data: "secret-token".to_string(),
    };

    assert_eq!(request.command_name(), "type_raw");
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
        states: Some(vec![TabState {
            tab_index: 6,
            name: "星河".to_string(),
            process_alive: true,
            last_output_ago_secs: 3,
            status: Some("running".to_string()),
            progress: Some(80),
            task: Some("查日志".to_string()),
            blocked_reason: None,
            last_report: None,
        }]),
        state: None,
    };

    let value = serde_json::to_value(&response).unwrap();

    assert_eq!(value["ok"], true);
    assert_eq!(value["status"], "received");
    assert!(value.get("error").is_none());
    assert!(value.get("target").is_none());
    assert_eq!(value["states"][0]["tab_index"], 6);
    assert_eq!(value["states"][0]["name"], "星河");
    assert_eq!(value["states"][0]["progress"], 80);
}
