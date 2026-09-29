---
id: IM-9
kind: work
group: 实现
title: 收敛业务状态与完成依据的唯一判定
status: done
completion_note: 业务状态与完成依据已收敛到 src/status.rs 的唯一判定：of() 负责状态归类（显式标注未验证的完成说明映射为 done_unverified），completion_gaps() 是完成门槛的唯一实现，check 与写入校验都调用它；show/list/summary 的状态呈现与过滤全部改用同一判定，不再各自解释 kind 声明。
completion_evidence: tests/status.rs 的两条对账测试（show 与 summary 对 todo/done/done_unverified/none 逐条一致；只有 status 字段、未声明完成映射的 kind 不再分叉）随 just ci 全绿（exit 0，66 个测试）。
---

## 目标

让一条记录的业务状态与完成依据只有一处判定：`show` 与 `summary` 读同一个来源，写入完成值的门槛只实现一次。

## 验收

- `show --json` 与 `summary --json` 对同一条记录给出同一个状态。
- 声明了名为 `status` 的字段、但没有声明 `completion_field` 的 kind 不再出现两个状态分叉。
- 完成门槛在 `checks.rs` 中写了两遍的实现收敛为一处。
- 为上述行为补一条对账断言（同一记录、两个命令、同一结果）。

## 当前下一步

在候选 A 的接口形状确定后动工；先把 `src/view.rs` 的状态判定与 `src/record/mod.rs` 的 `status()` 合并到一处。

## 历史进展

- 2026-09-28：架构评审（improve-codebase-architecture）列为 Strong 候选。依据：`src/view.rs:148-171`、`src/record/mod.rs:95-99`、`src/workspace/config.rs:54-67`、`src/checks.rs:261-292` 与 `306-334`、`src/cli/item.rs:91,153`、`src/cli/mod.rs:149-152`。测试夹具 kind `project-note` 已能暴露分叉。

## 证据

- 判定实现：src/status.rs（of、completion_gaps、is_unverified_note、writes_completion）。
- 对账测试：tests/status.rs。
- 架构评审报告已入库：docs/architecture-review-2026-09-28.html 锚点 #candidate-a（快照说明见该文件顶部横幅）。

## 进展

- 2026-09-28：已实现：新增 src/status.rs 承载唯一判定；view.rs 再导出 State，checks.rs 删除重复的完成门槛，cli 的 show/list/summary 改用同一判定；新增 --json 的 state 键。
