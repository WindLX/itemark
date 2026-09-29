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
    let matches = match command.clone().try_get_matches_from(args.clone()) {
        Ok(matches) => matches,
        Err(error) if error.use_stderr() => {
            if let Some(help) = help_after_parse_error(&command, &args) {
                print!("{help}");
                return;
            }
            let original = error.to_string();
            let mut message = itemark::i18n::localize_diagnostic(&original, language);
            if error.kind() == clap::error::ErrorKind::InvalidValue
                && original.contains("--status")
                && original.contains("but none was supplied")
            {
                message.push('\n');
                message.push_str(itemark::i18n::text("diag_status_missing_tip", language));
            }
            eprintln!("{}", message);
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

/// If Clap stopped at a missing value before an explicit help flag, retry the
/// help request while removing preceding option tokens one by one. This path
/// runs only after parsing failed and never executes the requested command.
fn help_after_parse_error(
    command: &clap::Command,
    args: &[std::ffi::OsString],
) -> Option<clap::Error> {
    let help_index = args
        .iter()
        .enumerate()
        .skip(1)
        .take_while(|(_, arg)| arg.to_string_lossy() != "--")
        .find(|(_, arg)| {
            let arg = arg.to_string_lossy();
            arg == "--help" || arg == "-h"
        })
        .map(|(index, _)| index)?;
    let mut help_args = args[..=help_index].to_vec();

    for index in (1..help_index).rev() {
        if args[index].to_string_lossy().starts_with('-') {
            help_args.remove(index);
            if let Err(error) = command.clone().try_get_matches_from(help_args.clone())
                && !error.use_stderr()
            {
                return Some(error);
            }
        }
    }
    None
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
