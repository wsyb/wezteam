use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericFilePath, GenericNamespaced, Name, Stream};

use crate::protocol::{Request, Response};

/// 读写超时
const TIMEOUT: Duration = Duration::from_secs(5);

/// 后端端点前缀（与后端 IpcConfig.endpoint_prefix 一致）
const ENDPOINT_PREFIX: &str = "TeamShell-wezteam";

/// 发现后端 IPC 端点名称
///
/// 优先级：
/// 1. 环境变量 HICLI_ENDPOINT（完整端点路径）
/// 2. 环境变量 HICLI_BACKEND_PID（后端进程 PID，构造端点路径）
/// 3. 固定名称 teamshell（与后端 IpcServer 一致）
fn discover_endpoint() -> String {
    if let Ok(endpoint) = std::env::var("HICLI_ENDPOINT") {
        return endpoint;
    }
    if let Ok(pid) = std::env::var("HICLI_BACKEND_PID") {
        return format!("{}-{}", ENDPOINT_PREFIX, pid);
    }
    ENDPOINT_PREFIX.to_string()
}

/// 构建本地套接字名称
///
/// 注意：会泄漏 endpoint 字符串以获得 'static 生命周期（CLI 进程生命周期内可接受）。
fn make_socket_name(endpoint: String) -> Result<Name<'static>, String> {
    let endpoint_str: &'static str = Box::leak(endpoint.into_boxed_str());
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

/// 发送请求到 teamshell 后端并返回响应
///
/// 通信协议：请求和响应各为一行 JSON（换行符分隔）
/// 连接失败时自动重试（指数退避，最多 3 次）
pub fn send_request(request: &Request) -> Result<Response, String> {
    let endpoint = discover_endpoint();
    let name = make_socket_name(endpoint.clone())?;

    let max_retries = 3;
    let mut delay = Duration::from_millis(100);
    let mut stream = None;

    for attempt in 0..max_retries {
        match Stream::connect(name.clone()) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) if attempt < max_retries - 1 => {
                eprintln!(
                    "连接失败，{}ms 后重试 ({}/{}): {}",
                    delay.as_millis(),
                    attempt + 1,
                    max_retries,
                    e
                );
                std::thread::sleep(delay);
                delay *= 2;
            }
            Err(e) => {
                return Err(format!(
                    "无法连接到 teamshell 后端 ({}): {}\n请确认后端正在运行。",
                    endpoint, e
                ));
            }
        }
    }

    let mut stream = stream.unwrap();

    let _ = stream.set_recv_timeout(Some(TIMEOUT));
    let _ = stream.set_send_timeout(Some(TIMEOUT));

    let request_json =
        serde_json::to_string(request).map_err(|e| format!("序列化请求失败: {}", e))?;

    stream
        .write_all(request_json.as_bytes())
        .map_err(|e| format!("发送请求失败: {}", e))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("发送请求分隔符失败: {}", e))?;
    stream
        .flush()
        .map_err(|e| format!("刷新写入缓冲区失败: {}", e))?;

    let mut reader = BufReader::new(&stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if response_line.is_empty() {
        return Err("后端返回了空响应".to_string());
    }

    serde_json::from_str(response_line.trim())
        .map_err(|e| format!("解析响应失败: {} (原始: {})", e, response_line.trim()))
}
