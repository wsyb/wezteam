use std::sync::Arc;

use mux::Mux;
use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt, BufReader};
use wezterm_mux_server_impl::teamshell::server::handle_stream;

#[tokio::test]
async fn handle_stream_reads_one_json_line_and_writes_handler_response() {
    Mux::set_mux(&Arc::new(Mux::new(None)));

    let (client, server) = duplex(1024);
    let mut client = BufReader::new(client);

    let server_task = tokio::spawn(async move { handle_stream(server).await });

    client.write_all(b"{\"cmd\":\"status\"}\n").await.unwrap();

    let mut response = String::new();
    client.read_line(&mut response).await.unwrap();

    server_task.await.unwrap().unwrap();
    assert_eq!(response, "{\"ok\":true,\"states\":[]}\n");
}
