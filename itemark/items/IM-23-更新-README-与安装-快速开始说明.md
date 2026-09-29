---
id: IM-23
kind: work
group: 实现
title: 更新 README 与安装/快速开始说明
status: done
parent: IM-22
completion_note: README已统一Itemark自举入口、源码用法、平台计划与当前发布状态；写明Unix/PowerShell安装卸载参数和skill安装路径。
completion_evidence: 目标和安装内容经实物核对：scripts/install.sh与uninstall.sh参数/默认prefix、PowerShell脚本参数、release-artifacts matrix；LZB报告Unix临时归档安装卸载通过，README明确Windows未实机运行且无公开下载资产。临时项目用target/debug/itemark执行init、add和check；14个文档文件的本地Markdown链接均可解析。未运行本轮Cargo测试。
---

## 目标

说明面向首次使用者的项目定位、构建/安装方式、平台支持与发布状态；安装信息须与发布脚本/产物一致，不声称尚不存在的发布或 crate。

## 验收

README 快速开始可照做；支持平台和安装命令经 LZB 发布脚本核实；文档链接有效。

## 当前下一步

无；本事项验收完成。

## 历史进展

## 证据

## 进展

2026-09-29: 按本轮文档与技能任务创建，实施前先核对现行 CLI 与负责发布的脚本/产物。
- 2026-09-29：按最终迁移后的Itemark路径、CLI帮助与init starter完成验收；详情见完成说明与证据。
