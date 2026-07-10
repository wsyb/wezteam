use std::sync::Arc;

use mux::Mux;
use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt, BufReader};
use wezterm_mux_server_impl::teamshell::handler::Handler;
use wezterm_mux_server_impl::teamshell::output;
use wezterm_mux_server_impl::teamshell::protocol::{IpcRequest, IpcResponse, TabState};
use wezterm_mux_server_impl::teamshell::server::handle_client;

fn with_empty_mux() {
    Mux::set_mux(&Arc::new(Mux::new(None)));
}

#[test]
fn handler_returns_ipc_response_for_missing_tab() {
    with_empty_mux();

    let response = Handler::handle(IpcRequest::Close { tab_index: 99 });

    assert_eq!(response.ok, false);
    assert_eq!(response.error.as_deref(), Some("tab 99 not found"));
}

#[test]
fn output_formats_status_response() {
    let response = IpcResponse {
        ok: true,
        error: None,
        status: None,
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

    let output = output::format_response(&response);
    assert!(output.contains("6 星河"));
    assert!(output.contains("[running]"));
    assert!(output.contains("80%"));
    assert!(output.contains("task=查日志"));
}

#[tokio::test]
async fn server_calls_handler_and_returns_real_response() {
    with_empty_mux();
    let (client, server) = duplex(1024);
    let mut client = BufReader::new(client);

    let token = "test-token";
    let server_task = tokio::spawn(async move { handle_client(server, token).await });

    client
        .write_all(b"{\"auth_token\":\"test-token\",\"cmd\":\"close\",\"tab_index\":99}\n")
        .await
        .unwrap();

    let mut response = String::new();
    client.read_line(&mut response).await.unwrap();

    server_task.await.unwrap().unwrap();
    assert_eq!(response, "{\"ok\":false,\"error\":\"tab 99 not found\"}\n");
}
