---
id: IM-27
kind: work
group: 实现
title: 准备 Itemark 发布产物与安全安装卸载
status: todo
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
completion_evidence: 当前 Cargo package 为 itemark 0.1.0，repository 为 https://github.com/WindLX/itemark；手动release-artifacts workflow仅上传限期workflow artifacts。公开Release资产尚不存在；LZB报告Unix临时归档安装/卸载测试通过，Windows PowerShell未本机实测。
---

## 目标

准备 Itemark 的MIT许可、Rust package元数据、跨目标预编译包与per-user安装/卸载。仅准备本地/CI产物，不实际对外发布。

## 验收

元数据包名与仓库信息为Itemark；手动打包目标与校验和覆盖Linux/macOS x86_64/aarch64及Windows x86_64；安装/卸载安全清单与skill profile支持。任何公开GitHub Release/tag/下载发布必须在全部当前与后续事项完成后，并先取得用户最终确认。Windows锁与脚本需按既有CI目标验证；不提前宣布资产可下载。

## 当前下一步

本事项等待当前批次及后续设计/功能事项IM-22、IM-28至IM-34收尾；最终外部发布另需用户确认。

## 历史进展

2026-09-29：创建发布准备事项；MIT 与目标矩阵已获用户确认（Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64），不 push、打 tag 或发布。

## 证据

当前 Cargo package 为 itemark 0.1.0，repository 为 https://github.com/WindLX/itemark；手动release-artifacts workflow仅上传限期workflow artifacts。公开Release资产尚不存在；LZB报告Unix临时归档安装/卸载测试通过，Windows PowerShell未本机实测。

## 进展

- 2026-09-29：确认当前锁实现直接对目录句柄调用 try_lock；Windows 支持尚待改用专用可写锁文件并在目标 CI 验证。
