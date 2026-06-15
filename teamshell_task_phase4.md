星河，新任务：Phase 4 — tsh CLI 输出格式优化

位置：D:\work\team2\tsh\src\commands.rs

当前 `output()` 函数（第5-11行）输出原始 JSON，需要改成纯文本格式。

修改 `output()` 函数：

```rust
fn output(response: &Response) {
    if !response.ok {
        eprintln!("Error: {}", response.error.as_deref().unwrap_or("unknown error"));
        std::process::exit(1);
    }

    if let Some(tabs) = &response.tabs {
        for tab in tabs {
            println!("{} {}", tab.index, tab.name);
        }
        return;
    }

    match response.status.as_deref() {
        Some("delivered") => {
            match response.target {
                Some(target) => println!("OK → {target}"),
                None => println!("OK"),
            }
        }
        Some("created") => {
            match (response.tab, response.text.as_deref()) {
                (Some(tab), Some(name)) if !name.is_empty() => println!("Created tab {tab} ({name})"),
                (Some(tab), _) => println!("Created tab {tab}"),
                _ => println!("Created"),
            }
        }
        Some("closed") => {
            match response.target {
                Some(target) => println!("Closed tab {target}"),
                None => println!("Closed"),
            }
        }
        Some("renamed") => {
            match (response.target, response.text.as_deref()) {
                (Some(target), Some(new_name)) => println!("Tab {target} renamed to \"{new_name}\""),
                (Some(target), _) => println!("Tab {target} renamed"),
                _ => println!("Renamed"),
            }
        }
        Some(status) => println!("{status}"),
        None => {
            if let Some(text) = &response.text {
                print!("{text}");
            } else {
                println!("OK");
            }
        }
    }
}
```

关键点：
1. 错误时用 `eprintln!` 输出到 stderr，exit(1)
2. see 命令的 text 内容用 `print!`（不加换行，内容本身已含换行）
3. 其他确认用 `println!`
4. tsh 源文件已有 `use crate::protocol::Response`，不用改导入

修改完后运行：
```bash
cd D:\work\team2\tsh
cargo check
```

完成后通知我。
