//! CLI 入口：解析参数、执行命令并把错误映射为退出码。

use clap::Parser;

use worklog::cli::args::Cli;

fn main() {
    let cli = Cli::parse();
    if let Err(error) = worklog::cli::run(cli) {
        eprintln!("{error}");
        std::process::exit(error.exit_code());
    }
}
