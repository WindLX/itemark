---
id: IM-5
kind: work
group: 实现
title: Manage groups and query records
status: done
parent: null
depends_on:
- IM-2
completion_note: group 管理与查询已实现并验证：group list/add/show，list/search 支持按 group、kind、ID 与文本过滤；ID 项目内唯一，跨 group 引用不复制。
completion_evidence: tests/group.rs、tests/query.rs；just ci 全绿。
---

# Manage groups and query records

## 目标

Provide basic group management and list/search across the project's native work, fact, and term records. Groups are a single-level mixed-kind organization; each record belongs to exactly one group, while its project-wide ID remains stable.

## 验收

- [ ] Create and view groups using the configured record layout; do not add nested groups or a default group assumption.
- [ ] List and search records by group, kind, ID, and supported text fields with text and JSON output.
- [ ] Verify that IDs are unique across the entire project, not only within a group.
- [ ] Keep cross-group references pointing to the original ID; do not duplicate records.
- [ ] Report behavior not actually exercised as unverified.

## 当前下一步

After IM-2 establishes record read/write, implement the smallest group list/create and query path.

## 历史进展

- 2026-09-28 [用户裁定]: Approved as IM-5; depends on IM-2. Groups are one-level, mixed-kind, and each item has one group. IDs are project-wide and remain unchanged when records move between groups. The exact query fields remain bounded by the supported record fields.

## 证据

- User-approved phase split and group model relayed in this conversation; checked 2026-09-28. No CLI behavior exists yet.

## 进展

- 2026-09-28：已实现并验证：group list/add/show，以及 list/search 按 group、kind、ID 与文本过滤；ID 在项目内唯一，跨 group 引用按 ID 指向原记录而不复制。对应集成测试在 tests/group.rs 与 tests/query.rs。
