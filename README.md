# Worklog

项目工作记录系统。目标是让人和 AI 用同一套记录继续已登记的工作；它不是长期记忆，也不是完整对话存档。

## 当前状态

当前仓库是 Rust 空项目，CLI 尚未实现。这里的 Markdown 文档先作为本项目的首个工作记录样例；未来 CLI 接管同一批事项文件和派生索引。

- [项目约定](AGENTS.md)：协作和改动原则。
- [需求与验收](docs/spec.md)：首期要解决的问题和范围。
- [领域词汇](CONTEXT.md)：核心术语。
- [设计与 CLI 草案](docs/design.md)：未来命令行的设计入口。
- [CLI 交互草案](docs/cli.md)：命令职责、记录行为和 AI 操作接口。
- [配置与记录示例](docs/examples.md)：项目 TOML、外置 Markdown 模板和 YAML+正文样例。
- [AI 协作流程草案](docs/ai-workflow.md)：将来 skill 可遵循的 Worklog 专属操作流程；尚未安装。
- [工作记录](docs/worklog/README.md)：本项目自身事项的入口。

事项 Markdown 是记录权威；索引、总账和交接摘要都是可重建的派生内容。新会话从项目路径进入，先看工作记录入口与摘要，再按稳定 ID 读事项详情。
