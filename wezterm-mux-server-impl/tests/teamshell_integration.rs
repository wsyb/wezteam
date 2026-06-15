use std::sync::Arc;

use mux::Mux;
use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt, BufReader};
use wezterm_mux_server_impl::teamshell::handler::Handler;
use wezterm_mux_server_impl::teamshell::output;
use wezterm_mux_server_impl::teamshell::protocol::{IpcRequest, IpcResponse, TabInfo};
use wezterm_mux_server_impl::teamshell::server::handle_stream;

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
fn output_formats_ipc_response_as_plain_text() {
    let response = IpcResponse {
        ok: true,
        error: None,
        status: None,
        target: None,
        tab: None,
        lines: None,
        text: None,
        tabs: Some(vec![TabInfo {
            index: 6,
            name: "星河".to_string(),
        }]),
    };

    assert_eq!(output::format_response(&response), "6 星河");
}

#[tokio::test]
async fn server_calls_handler_and_returns_real_response() {
    with_empty_mux();
    let (client, server) = duplex(1024);
    let mut client = BufReader::new(client);

    let server_task = tokio::spawn(async move { handle_stream(server).await });

    client
        .write_all(b"{\"cmd\":\"close\",\"tab_index\":99}\n")
        .await
        .unwrap();

    let mut response = String::new();
    client.read_line(&mut response).await.unwrap();

    server_task.await.unwrap().unwrap();
    assert_eq!(response, "{\"ok\":false,\"error\":\"tab 99 not found\"}\n");
}
