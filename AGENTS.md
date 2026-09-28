# 项目约定

- 本项目记录工作事项，不管理 AI 长期记忆，也不归档完整对话。
- 先读 [需求与验收](docs/spec.md)；术语以 [CONTEXT.md](CONTEXT.md) 为准，命令设计见 [docs/design.md](docs/design.md)，事项追踪规则见 [issue-tracker 配置](docs/agents/issue-tracker.md)。
- 每个事项用稳定 ID 标识，ID 不依赖文件路径；单事项 Markdown 是权威记录，索引可重建。
- 只记录已确认的重要决策；进展、验证结果和可复用证据按事实记录。
- 完成状态必须有完成说明，并附证据或明确说明尚未验证。完成不等于已验证。
- 保留废弃事项和更正轨迹；重开事项沿用原 ID 并写明原因。
- 设计保持直接：清晰接口、简单内部实现。没有当前需求就不加 traits、多后端抽象、框架或防御性检查。
- 本库人读内容默认简体中文（zh-CN）；命令名、ID、配置键和 JSON 字段保持稳定，不翻译。语言设置只改呈现，不自动重写既有记录。
- 提交信息使用 Conventional Commits：`<type>(<scope>): <中文简述>`；按改动选择 feat、fix、docs、refactor、test、chore 等 type，scope 写有意义的范围，每个提交只包含一个逻辑改动。
- 仅当用户要求测试，或行为改动需要验证时运行相应检查；不为文档变更运行 Rust 测试。
