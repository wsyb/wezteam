use mux::pane::Pane;
use mux::tab;
use mux::Mux;
use portable_pty::CommandBuilder;
use wezterm_term::TerminalSize;

use super::protocol::{IpcRequest, IpcResponse, TabInfo};

pub struct Handler;

impl Handler {
    pub fn handle(request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::Send {
                tab_index,
                message,
                from_tab_id,
            } => Self::send(tab_index, &format_message(&message, from_tab_id)),
            IpcRequest::SendRaw { tab_index, data } => Self::send_raw(tab_index, &data),
            IpcRequest::See {
                tab_index,
                line_count,
            } => Self::see(tab_index, Some(line_count)),
            IpcRequest::List => Self::list(),
            IpcRequest::Open {
                name,
                command,
                args,
                cwd,
                env,
            } => Self::open(&name, command, args, cwd, env),
            IpcRequest::Close { tab_index } => Self::close(tab_index),
            IpcRequest::Name {
                tab_index,
                new_name,
            } => Self::name(tab_index, &new_name),
        }
    }

    pub fn list() -> IpcResponse {
        let mux = Mux::get();
        let mut tabs: Vec<TabInfo> = Vec::new();

        for window_id in mux.iter_windows() {
            if let Some(win) = mux.get_window(window_id) {
                for tab in win.iter() {
                    let title = tab.get_title();
                    let tab_id = tab.tab_id();
                    tabs.push(TabInfo {
                        index: tab_id + 1,
                        name: if title.is_empty() {
                            format!("tab_{}", tab_id + 1)
                        } else {
                            title
                        },
                    });
                }
            }
        }

        tabs.sort_by_key(|t| t.index);
        IpcResponse::list(tabs)
    }

    pub fn send(target: usize, text: &str) -> IpcResponse {
        match Self::get_active_pane(target) {
            Some(pane) => match pane.send_paste(text) {
                Ok(()) => IpcResponse::delivered(target),
                Err(e) => IpcResponse::error(format!("send_paste failed: {e:#}")),
            },
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn send_raw(target: usize, data: &str) -> IpcResponse {
        match Self::get_active_pane(target) {
            Some(pane) => {
                let mut writer = pane.writer();
                if let Err(e) = writer.write_all(data.as_bytes()) {
                    return IpcResponse::error(format!("write to PTY failed: {e:#}"));
                }
                if let Err(e) = writer.flush() {
                    return IpcResponse::error(format!("flush PTY failed: {e:#}"));
                }
                IpcResponse::delivered(target)
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn see(target: usize, num_lines: Option<usize>) -> IpcResponse {
        match Self::get_active_pane(target) {
            Some(pane) => {
                let dims = pane.get_dimensions();
                let line_count = num_lines.unwrap_or(dims.viewport_rows) as isize;
                let end = dims.physical_top + dims.scrollback_rows as isize;
                let start = (end - line_count).max(0);

                let (_first_row, lines) = pane.get_lines(start..end);
                let rendered_lines = lines
                    .iter()
                    .map(|line| line.as_str().to_string())
                    .collect::<Vec<_>>();
                let text = rendered_lines.join("\n");

                IpcResponse::text(target, rendered_lines, text)
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn open(
        name: &str,
        command: Option<String>,
        args: Option<Vec<String>>,
        cwd: Option<String>,
        env: Option<Vec<(String, String)>>,
    ) -> IpcResponse {
        let mux = Mux::get();
        let domain = mux.default_domain();
        let size = TerminalSize::default();
        let predicted_id = tab::next_tab_id() + 1;
        let cmd = command.map(|command| build_command(name, command, args, cwd, env, predicted_id));

        let windows = mux.iter_windows();
        let window_id = windows
            .first()
            .copied()
            .or_else(|| Some(*mux.new_empty_window(None, None)));

        match window_id {
            Some(wid) => match smol::block_on(domain.spawn(size, cmd, None, wid)) {
                Ok(tab) => {
                    if !name.is_empty() {
                        tab.set_title(name);
                    }
                    IpcResponse::created(tab.tab_id() + 1, name.to_string())
                }
                Err(e) => IpcResponse::error(format!("open failed: {e:#}")),
            },
            None => IpcResponse::error("open failed: no available window".to_string()),
        }
    }

    pub fn close(target: usize) -> IpcResponse {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(target);

        match mux.get_tab(tab_id) {
            Some(tab) => {
                if let Some(pane) = tab.get_active_pane() {
                    pane.kill();
                }
                mux.remove_tab(tab_id);
                IpcResponse::closed(target)
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    pub fn name(target: usize, new_name: &str) -> IpcResponse {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(target);

        match mux.get_tab(tab_id) {
            Some(tab) => {
                tab.set_title(new_name);
                IpcResponse::renamed(target, new_name.to_string())
            }
            None => IpcResponse::error(format!("tab {target} not found")),
        }
    }

    fn get_active_pane(external_id: usize) -> Option<std::sync::Arc<dyn Pane>> {
        let mux = Mux::get();
        let tab_id = Self::id_to_tab_id(external_id);
        mux.get_tab(tab_id)?.get_active_pane()
    }

    fn id_to_tab_id(external_id: usize) -> usize {
        external_id.saturating_sub(1)
    }
}

fn get_tab_name(external_id: usize) -> String {
    let mux = Mux::get();
    let tab_id = external_id.saturating_sub(1);
    match mux.get_tab(tab_id) {
        Some(tab) => {
            let title = tab.get_title();
            if title.is_empty() {
                format!("tab_{external_id}")
            } else {
                title
            }
        }
        None => format!("tab_{external_id}"),
    }
}

fn format_message(message: &str, from_tab_id: Option<usize>) -> String {
    match from_tab_id {
        Some(from_tab_id) => {
            let name = get_tab_name(from_tab_id);
            format!("[TeamShell 消息] 来自 {name}:\n{message}\n")
        }
        None => format!("[TeamShell 消息] {message}\n"),
    }
}

fn build_command(
    name: &str,
    command: String,
    args: Option<Vec<String>>,
    cwd: Option<String>,
    env: Option<Vec<(String, String)>>,
    tab_id: usize,
) -> CommandBuilder {
    let mut builder = CommandBuilder::new(command);
    builder.env("TEAMSH_NAME", name);
    builder.env("TEAMSH_TAB_ID", tab_id.to_string());
    builder.env("TEAMSH_PLATFORM", std::env::consts::OS);

    if let Some(args) = args {
        builder.args(args);
    }
    if let Some(cwd) = cwd {
        builder.cwd(cwd);
    }
    if let Some(env_) = env {
        for (key, value) in env_ {
            builder.env(key, value);
        }
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_injects_env_vars() {
        let command = build_command("星河", "pwsh".to_string(), None, None, None, 3);

        assert_eq!(
            command.get_env("TEAMSH_NAME"),
            Some(std::ffi::OsStr::new("星河"))
        );
        assert_eq!(
            command.get_env("TEAMSH_TAB_ID"),
            Some(std::ffi::OsStr::new("3"))
        );
    }
}
