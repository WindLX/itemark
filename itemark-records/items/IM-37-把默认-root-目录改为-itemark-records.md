---
id: IM-37
kind: work
group: 实现
title: 把默认 root 目录改为 itemark-records
status: done
completion_note: 默认 root 改为 itemark-records 并同步迁移本仓库自举目录，随 v0.1.1 发布。
completion_evidence: 提交 f064b7d、92bd6b1；just ci 全绿；itemark check 36 条通过；新项目 init + add 端到端生成 itemark-records/。
---

## 目标

把 Itemark 默认数据 root 从 `itemark/` 改为 `itemark-records/`：`itemark/` 同时是命令名、仓库路径段和 `.agents/skills/itemark/` 的一部分，容易让人把数据目录误当成命令目录或仓库目录；同时把本仓库的自举 root 一起改名。

## 验收

- 省略 `root` 时默认 root 为 `itemark-records`；`itemark init` 在新项目创建 `itemark-records/items` 与 `itemark-records/templates`，并把 `root = "itemark-records"` 写进 `itemark.toml`。
- 已存在的项目不受影响：`init` 始终把 root 显式写进配置，root 由各项目的 `itemark.toml` 决定。
- 本仓库自举 root 迁到 `itemark-records/`，README、docs、tests、benches 与 release workflow 冒烟夹具中的 root 路径同步更新；`itemark.toml` 与 skill 目录名不变。
- `just ci` 全绿，本仓库 `itemark check` 通过，新项目 `init` + `add` 端到端可用。

## 当前下一步

无；改名、迁移与 v0.1.1 发布验收后关闭。

## 历史进展

- 2026-09-29：用户提出默认目录与命令同名的问题，在候选名里选定 `itemark-records`，并要求同步迁移本仓库、改完发布 v0.1.1。
- 选名理由：`.itemark/` 的隐藏目录与「记录是可见、可手改的 Markdown」取向冲突（编辑器与新人默认看不到，且 Windows 上点号不隐藏）；`itemark-records/` 不与命令、skill 目录或仓库路径冲突，也比通用的 `records/` 更不容易撞上项目既有目录。
- 实现：`git mv itemark itemark-records`；`src/workspace/config.rs` 的 `DEFAULT_ROOT` 改为 `itemark-records`；本仓库 `itemark.toml` 与全部 root 路径引用同步更新；历史迁移表 `docs/id-migrations/v1-wl-to-im.md` 按自身「路径是迁移时点快照」的约定保留原路径，只加改名说明。
- 版本与发布：`Cargo.toml` 升到 0.1.1，由 tag `v0.1.1` 触发 Release workflow 创建 Release。默认布局变化按语义化版本本应发 0.2.0，本次按用户明确要求发 0.1.1。

## 证据

- 提交 f064b7d `feat(config): 默认 root 目录改为 itemark-records`、92bd6b1 `chore(release): 版本升到 0.1.1`。
- `just ci`（fmt-check、check、clippy、test）全绿；本仓库 `itemark check` 输出「检查记录数：36 / 检查通过」。
- 新项目端到端：临时目录 `itemark init` 生成 `root = "itemark-records"` 与 `itemark-records/items`，`itemark add --kind work --group general --title 示例记录` 成功创建 IM-1，且未生成旧的 `itemark/` 目录。
