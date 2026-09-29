---
id: IM-3
kind: work
group: 验证
title: Verify v0 takeover and cross-session continuation
status: done
parent: null
depends_on:
- IM-4
- IM-8
completion_note: 五个验收条件均已在 2026-09-29 的陌生会话接续场景中实测通过；未验证清单与本次新增未验证项见进展与 docs/v0-adaptation.md 第 4 节。
completion_evidence: 独立只读会话的接续报告（入口顺序、执行命令、逐字引用与未记录项清单）；IM-1 迁移前后 sha256 核对；show/summary/handoff 的 JSON 键实测输出。
---

# Verify continuation and handoff summaries

## 目标

Use the implemented CLI to take over this repository's hand-maintained v0 records and verify that a new session can continue from the current overview and item details without relying on the original conversation.

## 验收

- [x] Inspect all existing v0 records and document the adaptation needed for the CLI; keep IM-1 byte-for-byte unchanged as explicitly grandfathered history, not as a final-format example. — 已满足：docs/v0-adaptation.md 给出格式契约、逐字段映射、八个 v0 文件逐一处置与接管命令序列；迁移前的 IM-1 与基线逐字节一致（sha256 0613ebfe…，见证据）。
- [x] The CLI can locate records by project-wide stable ID and expose each record's goal, acceptance, current state, next step, history, and evidence. — 已满足：仓库外用 `worklog show IM-3 --project <仓库> --json` 成功，返回 id/kind/group/title/status/state/parent/depends_on/fields/body/completion_note/completion_evidence/lifecycle/path；正文含目标、验收、当前下一步、历史进展、证据五节。
- [x] The generated overview and handoff identify source IDs and generation time and reflect current record state. — 已满足：`summary --json` 键为 counts/generated_at/groups/items/kinds/sources（sources 19 条），`summary --handoff --json` 键为 blocked/done_unverified/generated_at/in_progress/next_steps/sources；生成时点随当前记录刷新。
- [x] A fresh session can continue the unfinished work using the project entry point and cited records, with no inferred unrecorded completion. — 已满足：2026-09-29 由独立只读会话实测接续（见进展第 1 条），逐字引用记录内容并显式列出未记录/无法确认项，未臆测任何未记录的完成。
- [x] Report any compatibility or continuation behavior not exercised as unverified. — 已满足：未验证清单见 docs/v0-adaptation.md 第 4 节；本次场景覆盖其中「陌生会话接续」一项，其余保持标注。

## 当前下一步

After IM-4 and IM-8 are complete, run a fresh-session continuation scenario against the repository's v0 records and record the result.

## 历史进展

- 2026-09-28: Created as a planned acceptance item based on the user's approved cross-session handoff goal relayed in this conversation. No continuation scenario has been run.
- 2026-09-28 [决策]: 主要通过真实 CLI 操作临时项目目录验证外部行为；仅在有实际需要时增加少量函数测试，不 mock 存储。尚未实现或运行这些验证。
- 2026-09-28 [决策]: CLI 验证覆盖默认人读文本、可选 JSON、非零错误退出，以及定向更新保留正文的外部行为；尚未实现或运行这些验证。
- 2026-09-28 [用户裁定]: 本项依赖 IM-8（总览/交接）与 IM-4（真实 CLI 并发验收），负责用新 CLI 接管本仓 v0 手工记录并做陌生会话接续验收。
- 2026-09-28 [约束]: v0 记录待适配，不能作为新版格式的合法范例；IM-1 必须保持不变，按历史兼容样本处理。之前关于 kinds、group 与 facts/terms 状态“待确认”的口径已由后续裁定取代。

## 证据

- User-approved cross-session handoff goal relayed in this conversation; checked 2026-09-28. No continuation evidence exists yet.

## 进展

- 2026-09-28：v0 接管已执行并实测：新增 worklog.toml（root=worklog，groups 基线/实现/验证，kind work 声明 completion_field=status / completion_values=[done]），八条记录迁移到 worklog/items/，worklog check 报「检查记录数：8 / 检查通过」退出 0。逐字段映射与未验证清单见 docs/v0-adaptation.md。
- 2026-09-29：陌生会话接续场景已实测（独立只读会话，不共享本轮对话上下文）：按 AGENTS.md → README.md → worklog.toml → skills/worklog/SKILL.md → worklog/README.md → docs/ai-workflow.md → docs/agents/issue-tracker.md 定位入口，用 `./target/debug/worklog summary/list/show` 选中当时唯一待办事项 IM-3，逐字引用其目标、验收、状态、下一步、历史与证据，据 depends_on（IM-4、IM-8）均已 done 判定可开工，并显式列出「未记录/无法确认」项（验收勾选与进展叙述的对应、IM-1 是否保持不变、接续场景的通过判据）。
- 2026-09-29：该会话报出两处入口缺口并已修补：(1) README.md 未给构建与二进制位置 → 已补「运行方式」一行（cargo build / target/debug/worklog / cargo run -q --）；(2) 入口未说明如何判断可开工 → skills/worklog/SKILL.md「Locate and read」第 2 条已补「检查 depends_on 目标是否 complete，从未满足依赖的记录开始」。
- 2026-09-29：承接验收第 1 条，逐字节核对 IM-1：`git show 3579309:docs/worklog/IM-1.md` 与 `git show ace45c7^:docs/worklog/IM-1.md` 的 sha256 同为 0613ebfe6bd53004ab1df369ca9c625bb6630749063859d0fdfbca126d6940e9（接管前未被改动）；现行 worklog/items/IM-1.md（eadc1d93…）与 v0 原件仅差头部补 kind/group、标题加引号与正文末尾追加「## 完成说明」。
- 2026-09-29：迁移更正：从迁移前备份恢复本节两条被误改的历史，仅将可判定的稳定 ID 引用映射为 IM 编号；保留记录当时的 Worklog 品牌、worklog.toml、旧路径和 worklog CLI 命令。
