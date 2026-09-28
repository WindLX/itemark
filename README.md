# Worklog

项目工作记录系统。目标是让人和 AI 用同一套记录继续已登记的工作；它不是长期记忆，也不是完整对话存档。

## 当前状态

当前 CLI 已实现（`worklog` 二进制，仓库门禁为 `just ci`），本仓库用它自举管理自身事项：配置在 `worklog.toml`，活跃记录在 `worklog/items/`，模板在 `worklog/templates/`。下列 Markdown 文档承载需求、领域词汇与设计约定。

运行方式：在仓库内 `cargo build` 后使用 `target/debug/worklog <命令>`，或直接用 `cargo run -q -- <命令>`；把二进制装进 `PATH` 后，下文与文档里的 `worklog <命令>` 才能原样执行。

- [项目约定](AGENTS.md)：协作和改动原则。
- [需求与验收](docs/spec.md)：首期要解决的问题和范围。
- [领域词汇](CONTEXT.md)：核心术语。
- [设计与 CLI 草案](docs/design.md)：命令行的设计入口。
- [CLI 交互草案](docs/cli.md)：命令职责、记录行为和 AI 操作接口。
- [配置与记录示例](docs/examples.md)：项目 TOML、外置 Markdown 模板和 YAML+正文样例。
- [AI 协作流程草案](docs/ai-workflow.md)：将来 skill 可遵循的 Worklog 专属操作流程；尚未安装。
- [工作记录配置](worklog.toml)：本项目自身事项的入口；活跃记录在 `worklog/items/`，用 `worklog list` / `worklog show <ID>` 读取。
- [工作记录说明](worklog/README.md)：记录模型、状态与引用约定；v0 接管映射见 [v0 记录适配说明](docs/v0-adaptation.md)。

事项 Markdown 是记录权威；索引、总账和交接摘要都是可重建的派生内容。新会话从项目路径进入，先看工作记录入口与摘要，再按稳定 ID 读事项详情。

## 性能基准

`benches/worklog.rs` 用 [criterion](https://github.com/bheisler/criterion.rs) 覆盖写入路径（`add`：分配 ID、渲染模板、写盘、更新索引）、读取路径（`show` 的单条读取与 `list` 的索引扫描）以及 `check` 引用校验的规模增长（50 / 200 / 800 条记录）。

```text
cargo bench              # 全部基准
cargo bench -- add       # 只跑写路径
cargo bench -- read      # 只跑读取路径
cargo bench -- check     # 只跑引用校验
```

基准直接调用库 API 并在临时项目里自建夹具，因此不含 CLI 进程启动与参数解析成本；`just ci` 不运行基准，只编译它们。具体数值随机器变化，基准本身用于看同一台机器上代码改动前后的相对变化。
