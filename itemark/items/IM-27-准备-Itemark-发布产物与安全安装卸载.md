---
id: IM-27
kind: work
group: 实现
title: 准备 Itemark 发布产物与安全安装卸载
status: done
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
completion_evidence: 候选版本 0.1.0，准备提交 b83e0ebbdd56b3b9805091313f08a438a7996baa。跨平台 CI https://github.com/WindLX/itemmark/actions/runs/36532563339 五目标全部成功；产物 workflow https://github.com/WindLX/itemmark/actions/runs/36532563213 的版本匹配、五目标release build/package与native安装/卸载smoke、skill打包、SHA256SUMS和bundle上传全部成功。workflow提供7个临时artifacts，总包5.84 MB，保留30天，不是公开Release。cargo publish --dry-run --locked --registry crates-io已成功打包并明确中止上传；未发布crate。
completion_note: 发布准备验收完成：版本元数据、五目标构建与临时bundle、安装/卸载 smoke、skill 包和校验和均已在原生 CI 验证。未创建 tag、GitHub Release 或发布 crates.io；任何正式公开发布仍须等全部事项完成并取得用户明确确认。
---

## 目标

准备 Itemark 的MIT许可、Rust package元数据、跨目标预编译包与per-user安装/卸载。仅准备本地/CI产物，不实际对外发布。

## 验收

元数据包名与仓库信息为Itemark；手动打包目标与校验和覆盖Linux/macOS x86_64/aarch64及Windows x86_64；安装/卸载安全清单与skill profile支持。任何公开GitHub Release/tag/下载发布必须在全部当前与后续事项完成后，并先取得用户最终确认。Windows锁与脚本需按既有CI目标验证；不提前宣布资产可下载。

## 当前下一步

无。公开发布、tag或crates.io上传不属于本次已完成准备；须等所有事项完成并取得用户明确确认。

## 历史进展

2026-09-29：创建发布准备事项；MIT 与目标矩阵已获用户确认（Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64），不 push、打 tag 或发布。

## 证据

当前 Cargo package 为 itemark 0.1.0，repository 为 https://github.com/WindLX/itemark；手动release-artifacts workflow仅上传限期workflow artifacts。公开Release资产尚不存在；LZB报告Unix临时归档安装/卸载测试通过，Windows PowerShell未本机实测。

## 进展

- 2026-09-29：确认当前锁实现直接对目录句柄调用 try_lock；Windows 支持尚待改用专用可写锁文件并在目标 CI 验证。后续提交 b83e0eb 已修复 Windows 文件锁测试假设并通过 Windows CI。
- 2026-09-29：已核对 crates.io 官方 sparse index（itemark 路径当前 404）；Cargo include 白名单只保留源码、i18n、bench、清单、许可证和 crate 专用 README，cargo package 编译验证与本机 release 构建通过。
- 2026-09-29：核对根 README、Cargo package README、安装/卸载脚本与手动 artifact workflow；`cargo publish --dry-run --locked --registry crates-io` 成功打包并中止上传，未显示凭据或上传。该项重复记录已合并。
- 2026-09-29：提交 b83e0eb 的跨平台 CI 全部五目标成功；发布产物 workflow 五目标构建、安装/卸载 smoke、skill 包、版本一致性、SHA256SUMS 与 bundle 上传成功。仅生成保留30天的workflow artifacts，未tag、push公开发行、创建GitHub Release或发布crate。
