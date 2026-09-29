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
completion_note: 发布准备与首次正式发布完成：Release 由 tag 触发，五目标预编译包、skill 包、SHA256SUMS 与 GitHub Release 均已产出；安装/卸载脚本支持 curl 与 PowerShell 远程执行，卸载只删安装器自己放置的二进制与带标记的 skill。
completion_evidence: 提交 c94c236 改为 tag 触发发布、62adc96 增加在线安装/卸载、a4b3ce3 更新 README、0a2b075 修复 Windows 在线安装的校验和查找。首次 v0.1.0 tag 运行 https://github.com/WindLX/itemark/actions/runs/36536874373 因 Windows 在线安装 smoke 失败（SHA256SUMS is missing a checksum for itemark.zip）未创建 Release；修复后重推 tag，https://github.com/WindLX/itemark/actions/runs/36537457866 九项作业全部成功并创建 https://github.com/WindLX/itemark/releases/tag/v0.1.0（7 个资产：五目标二进制归档、itemark-skill-v0.1.0.zip、SHA256SUMS）。本机 Linux x86_64 实测 latest 在线安装（含 codex skill profile）→ 版本 0.1.0、skill 与 receipt 正确；在线卸载无残留且保留无关文件；发布页归档 sha256sum -c 校验成功。Windows 由 windows-latest runner 的归档安装/卸载与在线安装/卸载 smoke 实测通过。crates.io 未发布。
---

## 目标

准备 Itemark 的MIT许可、Rust package元数据、跨目标预编译包与per-user安装/卸载。仅准备本地/CI产物，不实际对外发布。

## 验收

元数据包名与仓库信息为Itemark；手动打包目标与校验和覆盖Linux/macOS x86_64/aarch64及Windows x86_64；安装/卸载安全清单与skill profile支持。任何公开GitHub Release/tag/下载发布必须在全部当前与后续事项完成后，并先取得用户最终确认。Windows锁与脚本需按既有CI目标验证；不提前宣布资产可下载。

## 当前下一步

无。tag 与 GitHub Release 已发布并实测；crates.io 发布仍单独授权，不与 Release 自动联动。

## 历史进展

2026-09-29：创建发布准备事项；MIT 与目标矩阵已获用户确认（Linux x86_64/aarch64、macOS x86_64/aarch64、Windows x86_64），不 push、打 tag 或发布。

## 证据

当前 Cargo package 为 itemark 0.1.0，repository 为 https://github.com/WindLX/itemark；手动release-artifacts workflow仅上传限期workflow artifacts。公开Release资产尚不存在；LZB报告Unix临时归档安装/卸载测试通过，Windows PowerShell未本机实测。
本地端到端验证（2026-09-29，Linux x86_64）：以 python3 静态服务器托管 itemark-v0.1.0-x86_64-unknown-linux-gnu.tar.gz、itemark-skill-v0.1.0.zip、SHA256SUMS 与两个安装脚本，`curl -fsSL $base/install.sh | sh -s -- --version 0.1.0 --base-url $base --prefix ... --skill-profile codex --skill-home ...` 安装成功且版本、skill、receipt 与无关文件断言全通过；在线卸载无残留；篡改校验和后安装中止且未留残留；`--base-url` 缺 `--version` 被拒；本地 `--archive` 安装/卸载通过。`sh -n` 与 js-yaml 校验 release.yml/ci.yml 通过。Windows 脚本本机无 pwsh，未实测。

## 进展

- 2026-09-29：确认当前锁实现直接对目录句柄调用 try_lock；Windows 支持尚待改用专用可写锁文件并在目标 CI 验证。后续提交 b83e0eb 已修复 Windows 文件锁测试假设并通过 Windows CI。
- 2026-09-29：已核对 crates.io 官方 sparse index（itemark 路径当前 404）；Cargo include 白名单只保留源码、i18n、bench、清单、许可证和 crate 专用 README，cargo package 编译验证与本机 release 构建通过。
- 2026-09-29：核对根 README、Cargo package README、安装/卸载脚本与手动 artifact workflow；`cargo publish --dry-run --locked --registry crates-io` 成功打包并中止上传，未显示凭据或上传。该项重复记录已合并。
- 2026-09-29：提交 b83e0eb 的跨平台 CI 全部五目标成功；发布产物 workflow 五目标构建、安装/卸载 smoke、skill 包、版本一致性、SHA256SUMS 与 bundle 上传成功。仅生成保留30天的workflow artifacts，未tag、push公开发行、创建GitHub Release或发布crate。
- 2026-09-29：新增交付范围：用户指出现有发布准备和本地归档安装不足，需增加正式 GitHub Release 与通过 curl/PowerShell 在线安装、卸载能力（latest 默认、可 pin 版本、可选 skill）。此前五平台临时打包、smoke、校验和及 cargo dry-run 验证仍然有效；本次是新增交付范围，不否定旧验证。正式 tag/Release 仍须用户最终确认；crates.io 发布单独授权，不与 Release 自动联动。
- 2026-09-29：发布流程改为 tag 驱动：新增 .github/workflows/release.yml（push v* tag 先校验 tag 版本与 Cargo.toml 一致，再创建 GitHub Release；workflow_dispatch 只打包验证），删除仅上传限期 artifacts 的 release-artifacts.yml。五目标构建后先在原生 runner 实测归档安装/卸载与在线安装/卸载，全部通过才创建 Release。
- 2026-09-29：安装/卸载脚本支持远程执行：install.sh / install.ps1 默认从 Release 取 latest，可用 --version / -Version 固定版本，下载归档与 SHA256SUMS 后校验（sha256sum/shasum/openssl 或 Get-FileHash），校验失败或中途出错回滚不留残留；uninstall.sh / uninstall.ps1 可直接 curl / irm 管道执行，按 receipt 只删安装器放置的二进制与带 .itemark-managed 标记的个人 skill，保留父目录与项目记录。
- 2026-09-29：安装器新增 --repo / --base-url（-Repo / -BaseUrl）与环境变量 ITEMARK_REPO / ITEMARK_BASE_URL / ITEMARK_VERSION / ITEMARK_PREFIX，base-url 用于镜像与测试且必须显式指定版本；根 README 增加「发布流程」「预编译安装（在线 curl / PowerShell）」章节，crates-io/README 增加预编译包安装入口。
- 2026-09-29：本地端到端验证通过：本地静态服务器托管与 Release 同构的资产（含 SHA256SUMS），curl 管道在线安装（含 codex skill profile）→ 断言二进制版本、skill、receipt 与保留无关文件 → 在线卸载后无残留 → 重装 → 篡改校验和时中止且不留残留 → --base-url 缺 --version 被拒 → 本地 --archive 安装/卸载。sh -n 与 js-yaml 校验 release.yml 均通过；Windows 脚本本机无 pwsh，未实测，由 workflow 的 windows-latest job 验证。
- 2026-09-29：v0.1.0 tag 首次推送后 Release workflow 失败：五个目标中 Windows x86_64-pc-windows-msvc 的「在线安装 smoke」在 install.ps1 校验步骤中止（SHA256SUMS is missing a checksum for itemark.zip）——远程模式把归档存成通用名，与 SHA256SUMS 里的发布资产名不一致；Unix 在线安装、四个非 Windows 目标与 Windows 归档安装/卸载 smoke 均通过。因为失败发生在 bundle 之前，未创建 GitHub Release，符合"全部通过才发布"的设计。修复：按发布资产名 itemark-v<版本>-<目标>.zip 与 itemark-skill-v<版本>.zip 落盘（提交 0a2b075），并把 v0.1.0 tag 移到该提交后重推。
- 2026-09-29：修复后重推 v0.1.0 tag，Release workflow 九项作业全部成功（https://github.com/WindLX/itemark/actions/runs/36537457866），已创建公开 Release https://github.com/WindLX/itemark/releases/tag/v0.1.0，含 7 个资产：五目标二进制归档（Linux/macOS x86_64 与 aarch64、Windows x86_64）、itemark-skill-v0.1.0.zip 与 SHA256SUMS。
- 2026-09-29：真实发布端到端复核（本机 Linux x86_64，走 raw.githubusercontent.com 上的 main 分支脚本）：`curl -fsSL .../install.sh | sh -s -- --prefix <tmp> --skill-profile codex --skill-home <tmp>` 以 latest 解析到 v0.1.0、下载并校验通过、装好二进制与 skill，`itemark --version` 输出 0.1.0；在线卸载无残留且保留无关文件；另下载发布页归档与 SHA256SUMS，`sha256sum -c` 对 x86_64-linux 归档校验成功。
