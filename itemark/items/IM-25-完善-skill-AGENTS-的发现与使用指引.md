---
id: IM-25
kind: work
group: 实现
title: 完善 skill/AGENTS 的发现与使用指引
status: done
parent: IM-22
completion_note: Canonical skill位于.agents/skills/itemark，CLI参考随skill包自包含；.claude/skills/itemark提供短项目入口，README给出复制安装方式，AGENTS链接至源skill。
completion_evidence: 核对Codex .agents/skills与Claude .claude/skills发现入口均为真实文件、无symlink；references/cli.md相对路径存在并随目录分发。README/AGENTS/skill相对链接有效；14个相关文档文件共0个失效本地链接。
---

## 目标

按 Codex 与 Claude 官方技能规范整理短主 skill、按需参考、仓内发现入口及自举约束。

## 验收

技能在声称支持的发现路径可发现；源目录与已安装状态说清；skill 依赖的使用参考可随分发访问。

## 当前下一步

无；本事项验收完成。

## 历史进展

## 证据

## 进展

2026-09-29: 按本轮文档与技能任务创建，实施前先核对现行 CLI 与负责发布的脚本/产物。
- 2026-09-29: 开始按现行代码更新文档与技能；实际 CLI 行为用临时项目目录核验。
- 2026-09-29：按最终迁移后的Itemark路径、CLI帮助与init starter完成验收；详情见完成说明与证据。
