use super::handler::Handler;
use super::protocol::IpcRequest;
use anyhow::Context as _;
use interprocess::local_socket::traits::tokio::Listener as _;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub fn spawn_server_thread() {
    std::thread::spawn(|| {
        match tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
        {
            Ok(runtime) => {
                if let Err(err) = runtime.block_on(start_server()) {
                    log::error!("TeamShell server stopped: {:#}", err);
                }
            }
            Err(err) => log::error!("failed to start TeamShell runtime: {:#}", err),
        }
    });
}

pub async fn start_server() -> anyhow::Result<()> {
    let listener = teamshell_ipc::create_listener()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    loop {
        let stream = listener
            .accept()
            .await
            .context("failed to accept TeamShell client")?;

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream).await {
                log::error!("TeamShell client error: {:#}", err);
            }
        });
    }
}

pub async fn handle_client<T>(stream: T) -> anyhow::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut stream = stream;
    let mut request_bytes = Vec::new();
    let bytes_read = read_json_line(&mut stream, &mut request_bytes).await?;

    if bytes_read == 0 {
        return Ok(());
    }

    let request: IpcRequest =
        serde_json::from_slice(&request_bytes).context("failed to decode TeamShell request")?;
    log::info!("TeamShell request: {}", request.command_name());

    let response = serde_json::to_string(&Handler::handle(request))
        .context("failed to encode TeamShell response")?;
    stream
        .write_all(response.as_bytes())
        .await
        .context("failed to write TeamShell response")?;
    stream
        .write_all(b"\n")
        .await
        .context("failed to write TeamShell response newline")?;
    stream
        .flush()
        .await
        .context("failed to flush TeamShell response")?;

    Ok(())
}

async fn read_json_line<T>(stream: &mut T, bytes: &mut Vec<u8>) -> anyhow::Result<usize>
where
    T: AsyncRead + Unpin,
{
    let mut byte = [0u8; 1];

    loop {
        let bytes_read = stream
            .read(&mut byte)
            .await
            .context("failed to read TeamShell request")?;
        if bytes_read == 0 {
            return Ok(bytes.len());
        }

        bytes.push(byte[0]);
        if byte[0] == b'\n' {
            return Ok(bytes.len());
        }
    }
}

pub async fn handle_stream<T>(stream: T) -> anyhow::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    handle_client(stream).await
}
