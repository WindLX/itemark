//! CLI 入口：解析参数、执行命令并把错误映射为退出码。

use clap::FromArgMatches;

use worklog::cli::args::{Cli, localized_command};
use worklog::style;

fn main() {
    #[cfg(unix)]
    restore_sigpipe();
    let matches = localized_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    if let Err(error) = worklog::cli::run(cli) {
        eprintln!("{}", style::paint(style::error(), &error.to_string()));
        std::process::exit(error.exit_code());
    }
}

/// `worklog list | head` 这类用法会提前关闭管道。Rust 默认忽略 `SIGPIPE`，于是
/// 写入失败会让 `println!` 抛 panic；恢复默认处置后进程像其他 Unix 工具一样安静退出。
#[cfg(unix)]
fn restore_sigpipe() {
    // SAFETY: `signal` 只替换信号处置，不读写内存、不引入别名。
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
