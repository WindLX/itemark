# 本地事项追踪配置

Worklog 使用项目现有工作记录作为本地 tracker。`docs/spec.md` 承载功能规格；经确认的实现工作沿用 `worklog/items/` 下的事项文件和稳定 ID（由 `worklog` CLI 读写），`depends_on` 表示 ticket 间的阻塞依赖。不要为同一工作另建 `.scratch/<feature>/issues/` 副本，也不使用外部 issue tracker。

本地事项没有 triage 标签或字段。`to-spec` 更新现有功能规格，不发布外部 issue，也不添加 `ready-for-agent` 标签。`to-tickets` 在方案和拆分获用户确认后，使用现有事项记录承接 tickets；依赖写入事项关系，不额外引入标签或 triage 字段。

项目用 `worklog.toml` 指定事项目录和少量查询默认值，配置优先级为 CLI 参数、项目配置、内置默认值。配置不得承载工作流规则。本仓库的配置声明一个 `work` kind 与三个 group（基线、实现、验证），精确字段含义见 `docs/cli.md`。
