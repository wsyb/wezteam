Mimo，新任务：改进消息发送格式

位置：D:\work\wezteam\wezterm-mux-server-impl\src\teamshell\handler.rs

当前 `format_message()` 函数（第169-174行）只显示 sender 的 tab ID 数字：

```
[TeamShell 消息] 来自 6:
hello
```

需要改成显示 sender 的名字（从 Mux 查表）：

```
[TeamShell 消息] 来自 星河:
hello
```

修改方案：
1. 在 `format_message()` 上面新增一个 `get_tab_name(tab_id: usize) -> String` 函数
2. 这个函数用 `Mux::get()` 获取 tab，再获取 tab title
3. 如果找不到 tab，fallback 回 `"tab_{tab_id}"`
4. 修改 `format_message()`，在 `from_tab_id` 分支里调用 `get_tab_name(from_tab_id)` 获取名字
5. ID 转换注意：from_tab_id 是外部 1-based ID，Mux.get_tab() 需要 0-based

参考 handler.rs 已有的 id_to_tab_id 和 list() 里的 tab.get_title() 用法。

完成后运行：
```bash
cd D:\work\wezteam
cargo check -p wezterm-mux-server-impl
cargo test -p wezterm-mux-server-impl teamshell::
```

完成后通知我。
