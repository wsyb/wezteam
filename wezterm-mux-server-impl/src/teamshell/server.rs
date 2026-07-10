use super::handler::Handler;
use super::protocol::IpcRequest;
use anyhow::Context as _;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const CLIENT_TIMEOUT_SECS: u64 = 30;
const MAX_BACKOFF_SECS: u64 = 30;
const MAX_RESTART_ATTEMPTS: u32 = 10;
const MAX_REQUEST_SIZE: usize = 1024 * 1024;

static RESTART_COUNT: AtomicU32 = AtomicU32::new(0);

fn ts_log(level: &str, msg: &str) {
    match level {
        "ERROR" => log::error!("[teamshell] {}", msg),
        "WARN" => log::warn!("[teamshell] {}", msg),
        _ => log::info!("[teamshell] {}", msg),
    }

    let path = config::RUNTIME_DIR.join("teamshell.log");
    if let Some(parent) = path.parent() {
        let _ = config::create_user_owned_dirs(parent);
    }
    if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&path) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(f, "[{}] {} {}", ts, level, msg);
    }
}

fn is_addr_in_use_error(err: &anyhow::Error) -> bool {
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        return io_err.kind() == std::io::ErrorKind::AddrInUse;
    }
    let err_str = format!("{err:#}");
    err_str.contains("Address already in use")
        || err_str.contains("normally permitted")
        || err_str.contains("EADDRINUSE")
}

pub fn spawn_server_thread() {
    std::thread::spawn(|| {
        let mut attempt = 0u32;
        loop {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_io()
                    .enable_time()
                    .build()
                {
                    Ok(runtime) => runtime.block_on(start_server()),
                    Err(err) => {
                        let msg = format!("failed to create TeamShell runtime: {err:#}");
                        ts_log("ERROR", &msg);
                        Err(anyhow::anyhow!("{}", msg))
                    }
                }
            }));

            attempt += 1;
            RESTART_COUNT.store(attempt, Ordering::Relaxed);

            match result {
                Ok(Ok(())) => {
                    ts_log("WARN", "TeamShell server exited (will restart)");
                }
                Ok(Err(ref err)) => {
                    if is_addr_in_use_error(err) {
                        ts_log(
                            "INFO",
                            "TeamShell backend already running on port 31415, skipping",
                        );
                        return;
                    }
                    ts_log(
                        "ERROR",
                        &format!("TeamShell server error: {err:#} (will restart)"),
                    );
                }
                Err(_) => {
                    ts_log("ERROR", "TeamShell server panicked (will restart)");
                }
            }

            if attempt >= MAX_RESTART_ATTEMPTS {
                ts_log(
                    "ERROR",
                    &format!(
                        "TeamShell server exceeded max restart attempts ({})",
                        MAX_RESTART_ATTEMPTS
                    ),
                );
                return;
            }

            let delay = 5u64
                .saturating_mul(1u64.saturating_add(attempt.min(5) as u64))
                .min(60);
            ts_log(
                "INFO",
                &format!("restarting in {delay}s (attempt {attempt})"),
            );
            std::thread::sleep(Duration::from_secs(delay));
        }
    });
}

pub async fn start_server() -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", teamshell_ipc::TEAMSH_PORT);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("failed to bind TeamShell TCP listener on {}", addr))?;

    ts_log(
        "INFO",
        &format!("server started on {} (pid {})", addr, std::process::id()),
    );

    let auth_token = teamshell_ipc::TEAMSH_AUTH_TOKEN.to_string();
    let mut consecutive_accept_errors = 0u32;

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                consecutive_accept_errors = 0;
                let token = auth_token.clone();
                tokio::spawn(async move {
                    if let Err(err) = handle_client(stream, &token).await {
                        ts_log("WARN", &format!("client error: {err:#}"));
                    }
                });
            }
            Err(err) => {
                consecutive_accept_errors += 1;
                ts_log(
                    "WARN",
                    &format!("accept error #{}: {err:#}", consecutive_accept_errors),
                );
                let shift = consecutive_accept_errors.min(8);
                let delay_ms = 100u64 << shift;
                let delay =
                    Duration::from_millis(delay_ms).min(Duration::from_secs(MAX_BACKOFF_SECS));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

pub async fn handle_client<T>(mut stream: T, expected_token: &str) -> anyhow::Result<()>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut request_bytes = Vec::new();

    let read_result = tokio::time::timeout(
        Duration::from_secs(CLIENT_TIMEOUT_SECS),
        read_json_line(&mut stream, &mut request_bytes),
    )
    .await;

    let bytes_read = match read_result {
        Ok(Ok(n)) => n,
        Ok(Err(err)) => return Err(err.context("read request failed")),
        Err(_) => {
            ts_log("WARN", "client read timeout");
            return Err(anyhow::anyhow!(
                "client read timeout ({}s)",
                CLIENT_TIMEOUT_SECS
            ));
        }
    };

    if bytes_read == 0 {
        return Ok(());
    }

    let value: serde_json::Value =
        serde_json::from_slice(&request_bytes).context("failed to parse TeamShell request")?;

    let auth_token = value
        .get("auth_token")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if auth_token != expected_token {
        ts_log("WARN", "auth failed: invalid token");
        let _ = stream.shutdown().await;
        return Err(anyhow::anyhow!("authentication failed"));
    }

    let mut command_value = value;
    if let Some(obj) = command_value.as_object_mut() {
        obj.remove("auth_token");
    }

    let request: IpcRequest =
        serde_json::from_value(command_value).context("failed to decode TeamShell request")?;

    let cmd_name = request.command_name();
    ts_log("INFO", &format!("request: {cmd_name}"));

    let response = serde_json::to_string(&Handler::handle(request))
        .context("failed to encode TeamShell response")?;

    let write_result = tokio::time::timeout(Duration::from_secs(CLIENT_TIMEOUT_SECS), async {
        stream
            .write_all(response.as_bytes())
            .await
            .context("failed to write response")?;
        stream
            .write_all(b"\n")
            .await
            .context("failed to write newline")?;
        stream.flush().await.context("failed to flush")?;
        Ok::<(), anyhow::Error>(())
    })
    .await;

    match write_result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(err),
        Err(_) => {
            ts_log("WARN", "client write timeout");
            Err(anyhow::anyhow!(
                "client write timeout ({}s)",
                CLIENT_TIMEOUT_SECS
            ))
        }
    }
}

async fn read_json_line<T>(stream: &mut T, bytes: &mut Vec<u8>) -> anyhow::Result<usize>
where
    T: AsyncRead + Unpin,
{
    let mut byte = [0u8; 1];

    loop {
        let n = stream
            .read(&mut byte)
            .await
            .context("failed to read TeamShell request")?;
        if n == 0 {
            return Ok(bytes.len());
        }

        bytes.push(byte[0]);
        if byte[0] == b'\n' {
            return Ok(bytes.len());
        }

        if bytes.len() > MAX_REQUEST_SIZE {
            return Err(anyhow::anyhow!("request too large (>{})", bytes.len()));
        }
    }
}
