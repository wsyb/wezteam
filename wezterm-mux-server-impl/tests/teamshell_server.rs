use std::sync::Arc;

use mux::Mux;
use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt, BufReader};
use wezterm_mux_server_impl::teamshell::server::handle_client;

#[tokio::test]
async fn handle_client_reads_one_json_line_and_writes_handler_response() {
    Mux::set_mux(&Arc::new(Mux::new(None)));

    let (client, server) = duplex(1024);
    let mut client = BufReader::new(client);

    let token = "test-token";
    let server_task = tokio::spawn(async move { handle_client(server, token).await });

    client
        .write_all(b"{\"auth_token\":\"test-token\",\"cmd\":\"status\"}\n")
        .await
        .unwrap();

    let mut response = String::new();
    client.read_line(&mut response).await.unwrap();

    server_task.await.unwrap().unwrap();
    assert_eq!(response, "{\"ok\":true,\"states\":[]}\n");
}

#[tokio::test]
async fn handle_client_rejects_invalid_token() {
    Mux::set_mux(&Arc::new(Mux::new(None)));

    let (client, server) = duplex(1024);
    let mut client = BufReader::new(client);

    let token = "correct-token";
    let server_task = tokio::spawn(async move { handle_client(server, token).await });

    client
        .write_all(b"{\"auth_token\":\"wrong-token\",\"cmd\":\"status\"}\n")
        .await
        .unwrap();

    let result = server_task.await.unwrap();
    assert!(result.is_err());
}
