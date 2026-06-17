mod commands;
mod ipc;
mod protocol;

use clap::{Parser, Subcommand};

fn parse_env(s: &str) -> Result<(String, String), String> {
    let pos = s.find('=').ok_or("环境变量格式应为 KEY=VALUE")?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// TeamShell CLI — Agent 间通信命令行接口
///
/// 通过 IPC 连接到 TeamShell 后端，实现 Agent 之间的消息传递和状态管理。
#[derive(Parser)]
#[command(name = "tsh", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出团队成员（编号、名字、状态）
    List,
    /// 在指定工位的终端上打字
    Type {
        /// 目标工位编号
        id: usize,
        /// 要敲的文本内容（使用 --key 时可省略）
        message: Option<String>,
        /// 不自动追加回车
        #[arg(long)]
        no_enter: bool,
        /// 发送按键值（支持 \r \n \t \0 \xNN 转义）
        #[arg(long)]
        key: Option<String>,
    },
    /// 查看指定工位的终端屏幕
    View {
        /// 目标工位编号
        id: usize,
        /// 读取行数（默认 50）
        #[arg(default_value_t = 50)]
        lines: usize,
    },
    /// 招一个新成员
    Open {
        /// 成员名字
        name: String,
        /// 要执行的命令
        command: String,
        /// 命令的参数
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
        /// 工作目录
        #[arg(long)]
        cwd: Option<String>,
        /// 环境变量 KEY=VALUE
        #[arg(long = "env", value_parser = parse_env)]
        env: Vec<(String, String)>,
        /// 自动用最佳 shell 包装命令
        #[arg(long)]
        auto_shell: bool,
        /// 创建后自动发送的入职消息
        #[arg(long)]
        init_prompt: Option<String>,
    },
    /// 让成员离开
    Close {
        /// 目标成员编号
        id: usize,
    },
    /// 给成员改名
    Name {
        /// 目标成员编号
        id: usize,
        /// 新名字
        new_name: String,
    },
    /// 初始化项目的 TeamShell 协议
    Init {
        /// 强制覆盖，不询问
        #[arg(short = 'y', long = "yes")]
        yes: bool,
        /// 预览变更，不写入
        #[arg(long)]
        dry_run: bool,
        /// 输出当前协议内容到 stdout
        #[arg(long)]
        show: bool,
        /// 额外写入协议的配置文件（仅 .md）
        #[arg(trailing_var_arg = true)]
        include: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::cmd_list(),
        Commands::Type { id, message, no_enter, key } => {
            commands::cmd_type(id, message.as_deref(), !no_enter, key.as_deref())
        }
        Commands::View { id, lines } => commands::cmd_view(id, lines),
        Commands::Open { name, command, args, cwd, env, auto_shell, init_prompt } => {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            commands::cmd_open(&name, Some(&command), &args_refs, cwd.as_deref(), &env, auto_shell, init_prompt.as_deref())
        }
        Commands::Close { id } => commands::cmd_close(id),
        Commands::Name { id, new_name } => commands::cmd_name(id, &new_name),
        Commands::Init { yes, dry_run, show, include } => {
            commands::cmd_init(yes, dry_run, show, &include)
        }
    }
}
