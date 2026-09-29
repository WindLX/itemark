# 项目约定

- Itemark 管理工作事项、事实与术语，可由项目自定义 kind；不管理 AI 长期记忆，也不归档完整对话。
- 先读 [需求与验收](docs/spec.md)；术语以 [CONTEXT.md](CONTEXT.md) 为准，命令约束见 [docs/design.md](docs/design.md) 和 [CLI 参考入口](docs/cli.md)，事项追踪规则见 [issue-tracker 配置](docs/agents/issue-tracker.md)。
- 本仓库已用 Itemark CLI 自举；AI 操作流程见 [docs/ai-workflow.md](docs/ai-workflow.md)，Codex skill 源文件位于 [.agents/skills/itemark/SKILL.md](.agents/skills/itemark/SKILL.md)。不要恢复手工 v0 记录例外。
- 每条记录用项目范围内唯一的稳定 ID 标识，ID 不依赖文件路径；单条目 Markdown 是权威记录，索引可重建。
- 只记录已确认的重要决策；进展、验证结果和可复用证据按事实记录。
- 仅当 kind 显式声明完成字段和值且写入该完成值时，以及运行 `check` 时，执行固定完成依据检查：非空完成说明，以及证据引用或明确的未验证说明；未声明时不应用此检查。事实核实不自动完成工作事项，术语不要求工作业务状态。
- 所有 kind 的条目都可废弃和恢复，保留原 ID、历史及废弃前适用的业务状态；工作事项业务重开沿用原 ID 并记录原因，与通用恢复分开。保留历史更正轨迹。
- 设计保持直接：清晰接口、简单内部实现。没有当前需求就不加 traits、多后端抽象、框架或防御性检查。
- 本库人读内容默认简体中文（zh-CN）；命令名、ID、配置键和 JSON 字段保持稳定，不翻译。语言设置只改呈现，不自动重写既有记录。
- 提交信息使用 Conventional Commits：`<type>(<scope>): <中文简述>`；按改动选择 feat、fix、docs、refactor、test、chore 等 type，scope 写有意义的范围，每个提交只包含一个逻辑改动。
- 仅当用户要求测试，或行为改动需要验证时运行相应检查；不为文档变更运行 Rust 测试。
