---
id: IM-22
kind: work
group: 实现
title: Itemark CLI、自举与文档基础
status: in_progress
---

## 目标

完成 Itemark CLI、自举入口与文档基础：统一产品名、配置和事项路径，落实当前约定的编号/文件名、终端呈现与排序行为，并让README、CLI/AI文档和技能说明与真实实现相符。

## 验收

当前批次的文档/技能项 IM-23、IM-24、IM-25 与品牌/编号项 IM-29、IM-30，以及终端输出项 IM-33、IM-34 均完成；新入口 itemark.toml、itemark/ 数据 root 与 ID 映射可由 CLI 检查。完成本批不等于发布；归档 IM-28、聚合 IM-31、正文稳定引用 IM-32 留在本批之后讨论/实现。

## 当前下一步

等待 IM-29、IM-30、IM-33、IM-34 收尾，再运行最终文档/CLI核对并完成本批。

## 历史进展

## 证据

## 进展

2026-09-29: 按用户授权开始本轮 README、文档、skill 与 AGENTS 更新；分解为 IM-23 至 IM-25。
- 2026-09-29: 已迁移 canonical skill 到 .agents/skills/worklog（真实文件），完整 CLI 参考随 skill 分发；新增 Claude Code 短入口和 docs/cli 仓内链接。
- 2026-09-29：用户已确认：本项目整体品牌改为 Itemark；本轮文档及 canonical skill 改用 Itemark 名称与 `.agents/skills/itemark` 真文件路径。CLI 命令名和配置入口为 itemark / itemark.toml；仓库重命名、remote 与发布脚本由 LZB 处理。CLI/Cargo 源码由 codex 处理，不改 Cargo/source/tests。用户选择 MIT；预定发布目标为 Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64，尚无发布资产。现有 worklog.toml、自举数据 root 和记录 ID 暂保持原样，收尾统一迁移。新 ID 仍为 WL-数字，不补零；新条目文件名采用 WL-数字-条目标题.md，旧事项 ID 也会迁移至该格式；不引入 groupcode。
- 2026-09-29：后续裁定覆盖前一条编号记录：项目和事项 ID 前缀采用 IM；新 ID 为 IM-数字且不补零，既有 WL-数字 也迁为 IM-数字。文件名用 IM-数字-条目标题.md；不引入 groupcode。文件名可随标题变更，ID 唯一稳定；批量改名待 Codex 给旧新映射预览后统一迁移。
