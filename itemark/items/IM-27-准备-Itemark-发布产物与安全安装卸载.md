---
id: IM-27
kind: work
group: 实现
title: 准备 Itemark 发布产物与安全安装卸载
status: in_progress
parent: IM-22
depends_on:
- IM-22
- IM-28
- IM-29
- IM-30
- IM-31
- IM-32
- IM-33
- IM-34
- IM-35
- IM-36
completion_evidence: Cargo package 为 itemark 0.1.0，元数据指向 crates-io/README.md。crates.io 当前未发布；`cargo publish --dry-run --locked --registry crates-io` 已成功打包编译并明确中止上传。手动 workflow_dispatch 构建 Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64 五目标，另产出独立 skill zip 和 SHA256SUMS，workflow artifacts 保留 30 天，不创建 GitHub Release。Unix 安装/卸载临时环境实测通过；Windows PowerShell 安装/卸载尚未本机实测。
---

## 目标

准备 Itemark 的MIT许可、Rust package元数据、跨目标预编译包与per-user安装/卸载。仅准备本地/CI产物，不实际对外发布。

## 验收

元数据包名与仓库信息为Itemark；手动打包目标与校验和覆盖Linux/macOS x86_64/aarch64及Windows x86_64；安装/卸载安全清单与skill profile支持。任何公开GitHub Release/tag/下载发布必须在全部当前与后续事项完成后，并先取得用户最终确认。Windows锁与脚本需按既有CI目标验证；不提前宣布资产可下载。

## 当前下一步

确认五目标构建产物与校验和齐全；补做 Windows 安装/卸载目标平台验证；待所有当前事项完成后，由用户决定是否发布。当前只准备流程，不 tag、push 或发布。

## 历史进展

2026-09-29：创建发布准备事项；MIT 与目标矩阵已获用户确认（Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64），不 push、打 tag 或发布。

## 证据

当前 Cargo package 为 itemark 0.1.0，repository 为 https://github.com/WindLX/itemark；手动release-artifacts workflow仅上传限期workflow artifacts。公开Release资产尚不存在；LZB报告Unix临时归档安装/卸载测试通过，Windows PowerShell未本机实测。

## 进展

- 2026-09-29：确认当前锁实现直接对目录句柄调用 try_lock；Windows 支持尚待改用专用可写锁文件并在目标 CI 验证。
- 2026-09-29：已核对 crates.io 官方 sparse index（itemark 路径当前 404）；Cargo include 白名单只保留源码、i18n、bench、清单、许可证和 crate 专用 README，cargo package 编译验证与本机 release 构建通过。跨平台资产脚本和 workflow 由 LZB 继续核验；尚未 tag、创建 GitHub Release 或发布 crate。
- 2026-09-29：核对根 README、Cargo package README、安装/卸载脚本与手动 artifact workflow；记录源码用户级安装/卸载、五目标及 skill 独立包、30 天 workflow artifacts、Unix 已测/Windows 未实测。`cargo publish --dry-run --locked --registry crates-io` 成功打包并中止上传；仍未创建 tag、push、GitHub Release 或发布 crate。
- 2026-09-29：cargo publish --dry-run --locked --registry crates-io 完成：仅构建并验证上传包，结尾为 aborting upload due to dry run；未读取/显示凭据或上传。
