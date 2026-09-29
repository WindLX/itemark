# Itemark

Itemark 是一个本地优先的工作事项、事实与术语追踪器。记录以可编辑的 Markdown 文件保存，稳定 ID 用于跨分组引用；项目可自定义记录类型和字段。

Itemark 是 CLI 工具，不是长期记忆服务，也不会上传项目记录。

## 安装

crate 发布后，可用 Rust 的 Cargo 安装：

```sh
cargo install itemark
```

也可从源码构建：

```sh
cargo install --path .
```

运行 `itemark --help` 查看命令。配置和记录格式说明见 [项目仓库](https://github.com/WindLX/itemark)。

## License

MIT；详见随 crate 提供的 `LICENSE` 文件。
