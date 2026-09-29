---
id: IM-26
kind: work
group: 实现
title: 切换项目语言与本地化CLI呈现
status: done
parent: IM-22
completion_note: i18n、系统locale初始化、Itemark命名、IM ID迁移和可读输出调整均已实现；迁移保留历史记录并更新其引用。
completion_evidence: just ci (2026-09-29)：43 单元测试、55 集成测试通过；fmt、check、clippy -D warnings 通过。`itemark check` 覆盖本仓32条迁移记录通过，kind check ok=true；旧新 ID 映射见 docs/id-migrations/v1-wl-to-im.md。
---

## 目标

移除 `--language`，常规帮助、错误与文本仅由项目配置 language 控制；init 与无配置入口使用系统 locale；提供中英文人读界面，同时保持JSON字段、ID、命令名稳定。

## 验收

静态zh-CN/en JSON文案目录；真实CLI覆盖项目语言优先、init系统locale写配置、无配置help、错误本地化、--language移除、JSON keys稳定与纯净；help/error与主要视图可读；相关测试和门禁通过。

## 当前下一步

无；后续归档、聚合与正文稳定引用另行讨论。

## 历史进展

## 证据

## 进展

- 2026-09-29：确认旧版CLI help固定简中且有全局--language；真实CLI红测失败后加入配置预读、系统locale和JSON词典，语言/help/query测试通过。
- 2026-09-29：移除--language；init新项目按system locale写配置并生成general/work/fact/term模板；真实CLI验证项目语言优先、check与完成依据，以及English章节校验保持稳定。
- 2026-09-29：统一包、库、CLI、config、root为Itemark；新ID为IM-数字并读取旧WL；新增标题安全文件名和改标题同步rename，旧ID查询/分配兼容测试通过。
- 2026-09-29：调整list/show/check/summary人读层级；长标题、状态/分组元数据换行，summary计数逐行，空可选字段隐藏；保留完整标题。
- 2026-09-29：将仓库入口迁到 itemark.toml，root 迁到 itemark/；32条记录ID由WL数字迁为IM数字，按ID重命名标题文件，并更新关联引用。生成docs/id-migrations/v1-wl-to-im.md保存映射；itemark check覆盖32条通过，kind check无问题。
