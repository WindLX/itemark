---
id: IM-30
kind: work
group: 实现
title: 统一 Itemark 品牌与 IM 编号前缀
status: done
parent: IM-22
completion_note: Itemark 名称、配置入口、CLI、skill 与文档已统一；未把计划中的发布资产描述为已发布。
completion_evidence: Cargo.toml/CLI 入口为 itemark；配置/数据为 itemark.toml 与 itemark/；README 指向 .agents/skills/itemark/ 且说明无预编译 Release 资产；GitHub 仓库已改名 WindLX/itemark。迁移对照见 docs/id-migrations/v1-wl-to-im.md。
---

## 目标

将本轮由我负责的 README、规范、CLI 参考和 AI skill 统一为 Itemark 品牌与 itemark 命令入口。

## 验收

所有新文档/skill 使用 Itemark/itemark；canonical skill 目录为 `.agents/skills/itemark/`，Claude 仓内短入口在 `.claude/skills/itemark/`。MIT 和计划发布目标为 Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64；无已发布资产时不得声称可下载。保留当前 `worklog.toml`、数据 root 与记录 ID 到协调迁移时点；此项不改代码/remote/发布脚本。

## 当前下一步

无；发布需单独准备真实资产并由用户确认。

## 历史进展

## 证据

- 2026-09-29：本地目录迁移证据：仓库从 `/home/windlx/App/worklog` 移至 `/home/windlx/App/itemark`；执行前确认目标不存在、源目录存在且没有 cargo/rustc 构建进程。移动后 branch=main、HEAD=001ee8ab、origin=git@github.com:WindLX/itemark.git，CLI检查34条记录通过。

## 进展

- 2026-09-29：Itemark/itemark 品牌已进入 Cargo 包与二进制、itemark.toml、README/规范/skill、GitHub 仓库及自举数据。发布文档明确目前无 GitHub Release 资产。
