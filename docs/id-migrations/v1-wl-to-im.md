# ID 迁移记录 v1：WL → IM

日期：2026-09-29。此表记录本仓库自举数据的集中迁移。迁移只改稳定 ID 前缀与数字补零形式、当前记录文件名和可判定的稳定 ID 引用；历史叙述保留，路径不再承担身份。旧新映射用于追溯 Git 历史，不作为运行时兼容数据库。

表中“新文件名”是迁移时的路径快照，标题后续变更可能使文件重命名；它不是当前路径目录。稳定 ID 映射才是权威，当前路径可通过项目索引按 ID 查找。

配置入口由 `worklog.toml` 改为 `itemark.toml`，记录 root 由 `worklog/` 改为 `itemark/`（v0.1.1 把默认 root 改名为 `itemark-records/`；本表按上一条约定保留迁移时点路径）。共迁移 32 条记录。

| 旧 ID | 新 ID | 新文件名 |
|---|---|---|
| `WL-0001` | `IM-1` | `itemark/items/IM-1-Establish-the-manual-worklog-baseline.md` |
| `WL-0002` | `IM-2` | `itemark/items/IM-2-Build-the-minimum-worklog-CLI.md` |
| `WL-0003` | `IM-3` | `itemark/items/IM-3-Verify-v0-takeover-and-cross-session-continuation.md` |
| `WL-0004` | `IM-4` | `itemark/items/IM-4-Verify-concurrent-CLI-writes-and-stale-update-conflicts.md` |
| `WL-0005` | `IM-5` | `itemark/items/IM-5-Manage-groups-and-query-records.md` |
| `WL-0006` | `IM-6` | `itemark/items/IM-6-Update-move-and-log-record-changes.md` |
| `WL-0007` | `IM-7` | `itemark/items/IM-7-Drop-restore-and-validate-records.md` |
| `WL-0008` | `IM-8` | `itemark/items/IM-8-Build-overview-and-handoff-views.md` |
| `WL-0009` | `IM-9` | `itemark/items/IM-9-收敛业务状态与完成依据的唯一判定.md` |
| `WL-0010` | `IM-10` | `itemark/items/IM-10-评审并取舍其余架构候选-B-E.md` |
| `WL-0011` | `IM-11` | `itemark/items/IM-11-消除写入路径的三次全量索引重建.md` |
| `WL-0012` | `IM-12` | `itemark/items/IM-12-把-check-的引用校验从-O-n²-降到-O-n.md` |
| `WL-0013` | `IM-13` | `itemark/items/IM-13-落实两轴代码评审的-16-项发现.md` |
| `WL-0014` | `IM-14` | `itemark/items/IM-14-交接摘要只纳入在飞事项.md` |
| `WL-0015` | `IM-15` | `itemark/items/IM-15-收敛字段类型词表与-kind-check-的发现呈现.md` |
| `WL-0016` | `IM-16` | `itemark/items/IM-16-让-cli-层不承载面向使用者的文案.md` |
| `WL-0017` | `IM-17` | `itemark/items/IM-17-集中默认分节名常量.md` |
| `WL-0018` | `IM-18` | `itemark/items/IM-18-给人读输出加彩色渲染并可关闭.md` |
| `WL-0019` | `IM-19` | `itemark/items/IM-19-用-criterion-把性能基准纳入仓库.md` |
| `WL-0020` | `IM-20` | `itemark/items/IM-20-优化-CLI-人读提示与帮助文案.md` |
| `WL-0021` | `IM-21` | `itemark/items/IM-21-本地化帮助脚手架并恢复诊断文本语言.md` |
| `WL-0022` | `IM-22` | `itemark/items/IM-22-完善-Worklog-用户文档与技能分发.md` |
| `WL-0023` | `IM-23` | `itemark/items/IM-23-更新-README-与安装-快速开始说明.md` |
| `WL-0024` | `IM-24` | `itemark/items/IM-24-对齐-CLI-文档与-AI-工作流说明.md` |
| `WL-0025` | `IM-25` | `itemark/items/IM-25-完善-skill-AGENTS-的发现与使用指引.md` |
| `WL-0026` | `IM-26` | `itemark/items/IM-26-切换项目语言与本地化CLI呈现.md` |
| `WL-0027` | `IM-27` | `itemark/items/IM-27-准备-Worklog-发布产物与安全安装卸载.md` |
| `WL-0028` | `IM-28` | `itemark/items/IM-28-讨论已完成条目的归档.md` |
| `WL-0029` | `IM-29` | `itemark/items/IM-29-更新事项-ID-与标题文件名约定.md` |
| `WL-0030` | `IM-30` | `itemark/items/IM-30-统一-Itemark-品牌与-IM-编号前缀.md` |
| `WL-0031` | `IM-31` | `itemark/items/IM-31-讨论过细事项的机械聚合.md` |
| `WL-0032` | `IM-32` | `itemark/items/IM-32-讨论正文稳定引用与引用健康检查.md` |
