use std::time::Duration;

pub const TEAMSH_PORT: u16 = 31415;
pub const TEAMSH_AUTH_TOKEN: &str = "teamshell-31415";

pub fn connect() -> Result<std::net::TcpStream, String> {
    connect_with_retries(3, Duration::from_millis(100))
}

pub fn connect_with_retries(
    max_retries: usize,
    initial_delay: Duration,
) -> Result<std::net::TcpStream, String> {
    let addr = format!("127.0.0.1:{}", TEAMSH_PORT);

    let mut delay = initial_delay;
    for attempt in 0..max_retries {
        match std::net::TcpStream::connect(&addr) {
            Ok(stream) => {
                stream
                    .set_nodelay(true)
                    .map_err(|e| format!("设置 TCP_NODELAY 失败: {}", e))?;
                return Ok(stream);
            }
            Err(e) if attempt < max_retries - 1 => {
                log::debug!(
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
                return Err(format!("无法连接到 TeamShell 后端 ({}): {}", addr, e));
            }
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_valid() {
        assert!(TEAMSH_PORT > 1024);
        assert!(!TEAMSH_AUTH_TOKEN.is_empty());
    }
}
