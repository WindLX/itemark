---
id: IM-7
kind: work
group: 实现
title: Drop, restore, and validate records
status: done
parent: null
depends_on:
- IM-6
completion_note: drop/restore 与记录校验已实现并验证：废弃与恢复保留 ID、历史与业务状态；check 校验必填字段、基础类型、枚举、必填非空分节、引用完整性，并对声明了完成映射的 kind 应用完成规则。
completion_evidence: tests/drop_restore.rs、tests/check.rs、tests/completion.rs；just ci 全绿。
---

# Drop, restore, and validate records

## 目标

Implement the shared record lifecycle and validate all records against their configured kind definitions. Lifecycle is separate from kind-specific business and verification fields.

## 验收

- [ ] Drop/deprecate a record without deleting its ID or history; restore the same ID and preserve its existing business state.
- [ ] Validate all records for required fields, basic value types, allowed enum values, and configured Markdown sections that must exist and contain non-whitespace text.
- [ ] For kinds that declare a completion field and value, require a nonempty completion explanation and either evidence or a clear unverified explanation when that value is present; kinds without this declaration do not receive the completion check.
- [ ] Keep kind rules declarative: no scripts, arbitrary condition language, custom status transitions, or workflow engine.
- [ ] Work workflow state remains distinct from generic lifecycle; fact verification is its own field, and terms have no business state.
- [ ] Ensure manual edits that leave a record marked complete but remove required completion evidence are reported by `check`.
- [ ] Report behavior not actually exercised as unverified.

## 当前下一步

After IM-6 provides focused updates and history append, add drop/restore and whole-record validation.

## 历史进展

- 2026-09-28 [用户裁定]: Approved as IM-7; depends on IM-6. Generic drop/restore preserves ID and history and does not reset business state. Work has four workflow states; fact verification remains independent; terms have no business state.
- 2026-09-28 [用户裁定]: Kind config can declare names, descriptions, templates, simple required/type/enum fields, and required nonempty Markdown sections. A fixed completion check applies only where the kind explicitly declares completion field/value; no general rule engine is allowed.
- 2026-09-28 [约束]: Add may create an item with a required value temporarily empty; `check` reports the missing value. There is no extra draft status. For completion-valued records, the fixed completion evidence rule applies from the first release.

## 证据

- User-approved kind, lifecycle, and completion rules relayed in this conversation; checked 2026-09-28. No CLI behavior exists yet.

## 进展

- 2026-09-28：已实现并验证：drop/restore 只改通用生命周期，保留 ID、历史与业务状态；check 校验必填字段、基础类型、枚举、必填非空分节、引用完整性，并对声明了完成字段与值的 kind 应用完成规则；kind 规则保持声明式。对应集成测试在 tests/drop_restore.rs、tests/check.rs、tests/completion.rs。
