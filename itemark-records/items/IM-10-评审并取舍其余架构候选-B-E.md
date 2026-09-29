---
id: IM-10
kind: work
group: 实现
title: 评审并取舍其余架构候选 B–E
status: done
completion_note: 四条候选全部给出结论：B（让「检查发现」成为一个概念）采纳，拆为 IM-15；C（一条记录只有一个投影）放弃——单条记录投影与全项目视图服务不同查询形状，合并只会放大载荷，而它原本要修的「status 与 state 来源不同」已由 IM-9 消除；D（cli 层不再承载文案与配置写法）部分采纳，文案漏出拆为 IM-16，放弃用 TOML 序列化重写配置；E（领域词汇与章节名没有归属）采纳最小收敛拆为 IM-17，放弃让分节名可按 kind 配置。
completion_evidence: '三条采纳项已建记录 IM-15 / IM-16 / IM-17（worklog check 17 条通过）。评审依据：docs/architecture-review-2026-09-28.html 的 #candidate-b…e，以及逐条复核的源码位置（config.rs:275、kind.rs:85-108、kind_cmd.rs:143；item.rs:462 与 view.rs:54；group.rs:66-75 与 config_cmd.rs:88,94；record/mod.rs:17-18、view.rs:118,127、args.rs:187）。'
---

## 目标

对架构评审的其余候选逐项决定采纳或放弃，并记录理由；采纳项拆成独立记录后在本条收尾。

## 验收

- 候选 B（让「检查发现」成为一个概念）、C（一条记录只有一个投影）、D（cli 层不再承载文案与配置写法）、E（领域词汇与章节名没有归属）各自有明确结论。
- 采纳的候选已拆成独立记录；放弃的候选写明放弃理由。

## 当前下一步

先完成候选 A，再评估 B 与 C——它们建立在「一条记录只有一个判定」之上，放在 A 之后代价更低。

## 历史进展

- 2026-09-28：架构评审给出评级，B/C/D 为 Worth exploring，E 为 Speculative。E 的删除测试通过：`ItemId` 删除后无行为消失（接口≈实现，`Record::id()` 已返回 `&str`）。

## 证据

- 架构评审报告（已入库，带快照说明横幅）：docs/architecture-review-2026-09-28.html，锚点 #candidate-b / #candidate-c / #candidate-d / #candidate-e。
- 采纳项：IM-15（候选 B）、IM-16（候选 D 的文案部分）、IM-17（候选 E 的最小收敛）。
- 放弃项与理由见本记录「历史进展」。

## 进展

- 2026-09-28：已完成取舍：B 采纳、C 放弃、D 部分采纳、E 最小收敛；采纳项拆分为 IM-15、IM-16、IM-17。
