use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name};

pub const ENDPOINT_PREFIX: &str = "TeamShell-wezteam";

pub fn socket_path() -> String {
    if let Ok(path) = std::env::var("TEAMSH_SOCKET_PATH") {
        return path;
    }
    if let Ok(dir) = std::env::var("WEZTERM_EXECUTABLE_DIR") {
        return format!("{}/.teamshell.sock", dir.trim_end_matches('/'));
    }
    format!("/tmp/{}.sock", ENDPOINT_PREFIX)
}

pub fn discover_endpoint() -> String {
    if let Ok(endpoint) = std::env::var("HICLI_ENDPOINT") {
        return endpoint;
    }
    if let Ok(pid) = std::env::var("HICLI_BACKEND_PID") {
        return format!("{}-{}", ENDPOINT_PREFIX, pid);
    }
    socket_path()
}

fn make_socket_name_inner(endpoint: &str) -> Result<Name<'static>, String> {
    let endpoint_str: &'static str = Box::leak(endpoint.to_string().into_boxed_str());
    if GenericNamespaced::is_supported() {
        endpoint_str
            .to_ns_name::<GenericNamespaced>()
            .map_err(|e| format!("创建套接字名称失败: {}", e))
    } else {
        endpoint_str
            .to_fs_name::<GenericFilePath>()
            .map_err(|e| format!("创建套接字名称失败: {}", e))
    }
}

pub fn make_socket_name(endpoint: &str) -> Result<Name<'static>, String> {
    make_socket_name_inner(endpoint)
}

pub fn make_default_socket_name() -> Result<Name<'static>, String> {
    make_socket_name_inner(&socket_path())
}

pub fn connect() -> Result<interprocess::local_socket::Stream, String> {
    let endpoint = discover_endpoint();
    let name = make_socket_name(&endpoint)?;
    interprocess::local_socket::Stream::connect(name)
        .map_err(|e| format!("无法连接到 teamshell 后端 ({}): {}", endpoint, e))
}

pub fn create_listener() -> Result<interprocess::local_socket::tokio::Listener, String> {
    let path = socket_path();
    let name = make_socket_name_inner(&path)?;

    if std::path::Path::new(&path).exists() {
        let _ = std::fs::remove_file(&path);
    }

    let listener = interprocess::local_socket::ListenerOptions::new()
        .name(name)
        .create_tokio()
        .map_err(|e| format!("failed to create TeamShell listener {}: {}", path, e))?;

    log::info!("TeamShell server listening on {}", path);
    Ok(listener)
}
