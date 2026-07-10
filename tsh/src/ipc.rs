use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use crate::protocol::{Request, Response};

const TIMEOUT: Duration = Duration::from_secs(5);

pub fn send_request(request: &Request) -> Result<Response, String> {
    let mut stream = teamshell_ipc::connect_with_retries(3, Duration::from_millis(100))?;
    let _ = stream.set_read_timeout(Some(TIMEOUT));
    let _ = stream.set_write_timeout(Some(TIMEOUT));

    let mut value = serde_json::to_value(request).map_err(|e| format!("序列化请求失败: {}", e))?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "请求序列化结果不是 JSON 对象".to_string())?;
    obj.insert(
        "auth_token".to_string(),
        serde_json::Value::String(teamshell_ipc::TEAMSH_AUTH_TOKEN.to_string()),
    );
    let request_json =
        serde_json::to_string(&value).map_err(|e| format!("序列化请求失败: {}", e))?;

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
