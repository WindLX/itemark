//! CLI 入口：先确定呈现语言，再解析参数并运行命令。

use clap::FromArgMatches;
use itemark::cli::args::{Cli, localized_command};
use itemark::style;

fn main() {
    #[cfg(unix)]
    restore_sigpipe();
    let args: Vec<_> = std::env::args_os().collect();
    let language = itemark::i18n::language_for_args(args.get(1..).unwrap_or_default());
    let command = localized_command(language);
    let matches = match command.clone().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) if error.use_stderr() => {
            eprintln!(
                "{}",
                itemark::i18n::localize_diagnostic(&error.to_string(), language)
            );
            std::process::exit(2);
        }
        Err(error) => {
            print!("{error}");
            return;
        }
    };
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    if let Err(error) = itemark::cli::run(cli) {
        let message = itemark::i18n::localize_diagnostic(&error.to_string(), language);
        eprintln!("{}", style::paint(style::error(), &message));
        std::process::exit(error.exit_code());
    }
}

/// `itemark list | head` 这类用法会提前关闭管道。Rust 默认忽略 `SIGPIPE`，于是
/// 写入失败会让 `println!` 抛 panic；恢复默认处置后进程像其他 Unix 工具一样安静退出。
#[cfg(unix)]
fn restore_sigpipe() {
    // SAFETY: `signal` 只替换信号处置，不读写内存、不引入别名。
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
