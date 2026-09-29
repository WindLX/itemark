# 本地事项追踪配置

Itemark 使用项目现有工作记录作为本地 tracker。`docs/spec.md` 承载功能规格；经确认的实现工作沿用当前自举 root `itemark-records/items/` 下的事项文件和稳定 ID（由 `itemark` CLI 读写），`depends_on` 表示 ticket 间的阻塞依赖。ID 与文件名迁移对照见 [v1 迁移表](../id-migrations/v1-wl-to-im.md)。不要为同一工作另建 `.scratch/<feature>/issues/` 副本，也不使用外部 issue tracker。

本地事项没有 triage 标签或字段。`to-spec` 更新现有功能规格，不发布外部 issue，也不添加 `ready-for-agent` 标签。`to-tickets` 在方案和拆分获用户确认后，使用现有事项记录承接 tickets；依赖写入事项关系，不额外引入标签或 triage 字段。

新项目用 `itemark.toml` 指定事项目录和少量查询默认值，配置优先级为 CLI 参数、项目配置、内置默认值。配置不得承载工作流规则。本仓库也使用 `itemark.toml`；事项模型与 CLI 行为见 `docs/cli.md`。
