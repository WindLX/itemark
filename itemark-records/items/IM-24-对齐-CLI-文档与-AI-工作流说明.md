---
id: IM-24
kind: work
group: 实现
title: 对齐 CLI 文档与 AI 工作流说明
status: done
parent: IM-22
completion_note: CLI/AI工作流文档已对齐itemark.toml、IM数字ID和标题文件名、starter kind/init行为、语言配置及CLI操作参数；正文稳定引用仍明确标为未实现/后续讨论。
completion_evidence: 核对target/debug/itemark的根、init、add、show、list、search、update、log、drop、restore、group、kind、check、summary帮助；flags与references/cli.md一致。新建临时itemark.toml后kind check/check/add/list均退出0，分配IM-1并生成标题化文件。英文locale init写入Goal/Acceptance，改变配置语言后kind check通过，验证既有章节不自动转换；现有config执行init --force后root和内容不变。docs/examples TOML解析并用初始化生成的模板运行kind check/check/add/list成功。
---

## 目标

核对当前 CLI help 与实现，更新唯一权威 CLI 参考及 AI 工作流入口，保持说明与实现一致。

## 验收

命令、参数、默认值、错误行为与当前 CLI help/实现相符；无重复手维护长参数表。

## 当前下一步

无；本事项验收完成。

## 历史进展

## 证据

## 进展

2026-09-29: 按本轮文档与技能任务创建，实施前先核对现行 CLI 与负责发布的脚本/产物。
- 2026-09-29: 开始按现行代码更新文档与技能；实际 CLI 行为用临时项目目录核验。
- 2026-09-29：按最终迁移后的Itemark路径、CLI帮助与init starter完成验收；详情见完成说明与证据。
