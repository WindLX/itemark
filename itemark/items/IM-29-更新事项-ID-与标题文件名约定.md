---
id: IM-29
kind: work
group: 实现
title: 更新事项 ID 与标题文件名约定
status: done
parent: IM-22
completion_note: ID 与标题文件名规则已写入本仓规范，且迁移映射已提供 Git 历史追溯。
completion_evidence: docs/examples.md 明确 IM-数字不补零、ID 稳定而文件名随标题变化；docs/id-migrations/v1-wl-to-im.md 列出32条旧新ID和新文件名。`itemark check` 覆盖32条记录通过。
---

## 目标

在本批自举文档中记录已确认的稳定 ID 与文件名规则，并配合代码迁移方案核实命令行为。

## 验收

新 ID 使用 `IM-数字`，不补零；既有记录的 `WL-数字` 前缀也迁为 `IM-数字`。新文件名采用 `IM-数字-条目标题.md`；ID 才是稳定身份，标题改变可改文件名，既有迁移先展示旧新映射预览后执行。本事项只维护文档，不改源代码或自行迁移记录。

## 当前下一步

无；此映射用于追溯迁移前 Git 历史。

## 历史进展

## 证据

## 进展

- 2026-09-29：文档已统一 IM-数字不补零与标题可读文件名；迁移映射表已执行并保留旧新对应。
