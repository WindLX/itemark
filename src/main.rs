//! CLI 入口：解析参数、执行命令并把错误映射为退出码。

use clap::FromArgMatches;

use worklog::cli::args::{Cli, localized_command};
use worklog::style;

fn main() {
    let matches = localized_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    if let Err(error) = worklog::cli::run(cli) {
        eprintln!("{}", style::paint(style::error(), &error.to_string()));
        std::process::exit(error.exit_code());
    }
}
