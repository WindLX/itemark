---
id: IM-33
kind: work
group: 实现
title: 修复条目 ID 的数值排序
status: done
parent: IM-22
completion_note: 数值 ID 排序与状态展示回归已修复；真实 CLI 验收和完整 CI 通过。
completion_evidence: just ci：43 单元测试、55 集成测试通过；cargo fmt、cargo check --all-targets、cargo clippy --all-targets -D warnings 均通过。定向 CLI 测试覆盖 IM-1、IM-2、IM-10、IM-11；itemark check IM-33 通过。
---

## 目标

修复无补零 IM 数字 ID 引入的字典序回归；任何默认按 ID 展示或作为相同分数次级键的结果按 ID 数值排序。

## 验收

用真实CLI验证 `list` 默认按ID数值顺序呈现IM-1、IM-2、IM-10、IM-11；`search` 匹配分数相同时按ID数值作次级顺序；summary/handoff中的事项列表与sources按ID数值顺序。无需新增排序命令/参数，不改变已有非ID排序语义或任意数组顺序。

## 当前下一步

无；数值排序修复及验收已完成。

## 历史进展

## 证据

## 进展

- 2026-09-29：用户明确要求修复无补零 ID 的数值排序；查重无结果，已创建本事项并交由 Codex 实现。
- 2026-09-29：加入真实 CLI 排序回归：list 默认/分组筛选、search 同分、summary/handoff items 与 sources 均按数值排序；IM-11 后 add 分配 IM-12。完整 just ci 通过。
