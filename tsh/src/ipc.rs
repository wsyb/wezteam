use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use interprocess::local_socket::traits::Stream as _;

use crate::protocol::{Request, Response};

const TIMEOUT: Duration = Duration::from_secs(5);

pub fn send_request(request: &Request) -> Result<Response, String> {
    let endpoint = teamshell_ipc::discover_endpoint();
    let name = teamshell_ipc::make_socket_name(&endpoint)?;

    let max_retries = 3;
    let mut delay = Duration::from_millis(100);
    let mut stream = None;

    for attempt in 0..max_retries {
        match interprocess::local_socket::Stream::connect(name.clone()) {
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
