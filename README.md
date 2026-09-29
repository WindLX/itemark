# Itemark

项目工作记录系统。目标是让人和 AI 用同一套记录继续已登记的工作；它不是长期记忆，也不是完整对话存档。

## 当前状态

Itemark CLI 已实现（`itemark` 二进制，仓库门禁为 `just ci`），本仓库用它自举管理自身事项，配置入口为 `itemark.toml`、数据目录为 `itemark/`。目前尚未发布预编译安装包；下列 Markdown 文档承载需求、领域词汇与设计约定。

从源码运行：安装 Rust 工具链后，在仓库内运行 `cargo build --release`，再执行 `target/release/itemark <命令>`；开发时也可用 `cargo run -q -- <命令>`。预编译安装包尚未发布，不要把 crate 名或未来下载地址当作已提供的安装方式。

## 预编译安装

GitHub Release 资产尚未发布。手动 release-artifacts workflow 只生成限期保存的工作流产物，不代表已发布版本；拿到归档后可用仓库脚本安装：

```sh
# 将路径替换为已下载的 Unix 归档；默认安装到 ~/.local/bin/itemark
./scripts/install.sh --archive /path/to/itemark-v0.1.0-x86_64-unknown-linux-gnu.tar.gz

# 可选：同时安装 skill 包到 Codex 个人 skill 目录
./scripts/install.sh --archive /path/to/itemark-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  --skill-archive /path/to/itemark-skill-v0.1.0.zip --skill-profile codex

# 卸载该 prefix 下由脚本安装的文件
./scripts/uninstall.sh
```

Windows 使用 `scripts/install.ps1 -Archive <ZIP路径>` 和 `scripts/uninstall.ps1`；默认安装到 `$LOCALAPPDATA/Programs/Itemark`。安装器可用 `-SkillArchive <ZIP路径> -SkillProfile codex|claude` 同装 skill。计划目标为 Linux `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`、macOS `x86_64-apple-darwin` / `aarch64-apple-darwin`、Windows `x86_64-pc-windows-msvc`。

Unix 安装器还接受 `--prefix DIR`（默认 `$HOME/.local`）和可选 `--skill-home DIR`；卸载器也接受 `--prefix DIR`。PowerShell 安装/卸载器对应 `-Prefix DIR`，可选 `-SkillHome DIR`；Unix 本地归档安装和卸载已由发布负责人在临时环境验证，Windows 脚本尚未实机运行。公开发布需等当前工作完成后再由用户确认。

仓库文档和 CLI 人读提示默认使用简体中文（zh-CN）。项目可在配置中显式设置输出语言；现有记录不会因此被翻译或重写。

- [项目约定](AGENTS.md)：协作和改动原则。
- [需求与验收](docs/spec.md)：首期要解决的问题和范围。
- [领域词汇](CONTEXT.md)：核心术语。
- [设计说明](docs/design.md)：命令行的设计约束。
- [CLI 参考入口](docs/cli.md)：完整行为参考与当前 help 的位置。
- [许可证](LICENSE)：MIT。
- [配置与记录示例](docs/examples.md)：项目 TOML、外置 Markdown 模板和 YAML+正文样例。
- [AI 协作流程](docs/ai-workflow.md)：本项目使用 Itemark CLI 的步骤。
- [Itemark skill](.agents/skills/itemark/SKILL.md)：Codex 的仓内发现目录与唯一 skill 源文件；Claude Code 入口见 `.claude/skills/itemark/SKILL.md`。
- [当前自举配置](itemark.toml)：本项目事项入口；活跃记录在 `itemark/items/`，用 `itemark list` / `itemark show <ID>` 读取。
- [工作记录说明](itemark/README.md)：记录模型、状态与引用约定；v0 接管映射见 [v0 记录适配说明](docs/v0-adaptation.md)。
- [ID/文件名迁移映射](docs/id-migrations/v1-wl-to-im.md)：旧 WL 编号与新 IM 编号、标题文件名的对照。

将 skill 安装到个人配置时，把 `.agents/skills/itemark/` 整个目录复制到 Codex 的 `~/.agents/skills/` 或 Claude Code 的 `~/.claude/skills/`。仓内 Claude 入口引用 `.agents` 下的源文件；独立安装时请复制完整 skill 目录，不依赖仓库文件。

```sh
# Codex
mkdir -p ~/.agents/skills
cp -R .agents/skills/itemark ~/.agents/skills/

# Claude Code
mkdir -p ~/.claude/skills
cp -R .agents/skills/itemark ~/.claude/skills/
```

Windows PowerShell 可将 `.agents/skills/itemark` 复制到 `$HOME\.agents\skills\` 或 `$HOME\.claude\skills\`。卸载时删除对应配置目录中的 `itemark` 子目录；仓内源文件不受影响。二进制安装包与 skill 分开打包；目前尚无公开下载资产。

事项 Markdown 是记录权威；索引、总账和交接摘要都是可重建的派生内容。新会话从项目路径进入，先看工作记录入口与摘要，再按稳定 ID 读事项详情。

## 性能基准

`benches/itemark.rs` 用 [criterion](https://github.com/bheisler/criterion.rs) 覆盖写入路径（`add`：分配 ID、渲染模板、写盘、更新索引）、读取路径（`show` 的单条读取与 `list` 的索引扫描）以及 `check` 引用校验的规模增长（50 / 200 / 800 条记录）。

```text
cargo bench              # 全部基准
cargo bench -- add       # 只跑写路径
cargo bench -- read      # 只跑读取路径
cargo bench -- check     # 只跑引用校验
```

基准直接调用库 API 并在临时项目里自建夹具，因此不含 CLI 进程启动与参数解析成本；`just ci` 不运行基准，只编译它们。具体数值随机器变化，基准本身用于看同一台机器上代码改动前后的相对变化。
