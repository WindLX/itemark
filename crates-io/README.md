# Itemark

Itemark 是一个本地优先的工作事项、事实与术语追踪器。记录以可编辑的 Markdown 文件保存，稳定 ID 用于跨分组引用；项目可自定义记录类型和字段。

Itemark 是 CLI 工具，不是长期记忆服务，也不会上传项目记录。

## 安装

crate 发布到 crates.io 后，可用 Rust 的 Cargo 安装：

```sh
cargo install itemark
```

也可从源码构建：

```sh
cargo install --path .
```

不想装 Rust 工具链时，可安装 GitHub Release 上的预编译包（Linux / macOS 用安装脚本，Windows 用 PowerShell），默认安装最新版本：

```sh
curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.ps1 | iex
```

固定版本、指定安装目录或同时安装 AI skill 的选项，见[项目仓库](https://github.com/WindLX/itemark) README 的「预编译安装」一节。

运行 `itemark --help` 查看命令。配置和记录格式说明见 [项目仓库](https://github.com/WindLX/itemark)。

## License

MIT；详见随 crate 提供的 `LICENSE` 文件。
