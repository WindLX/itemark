# Itemark

项目工作记录系统。目标是让人和 AI 用同一套记录继续已登记的工作；它不是长期记忆，也不是完整对话存档。

## 当前状态

Itemark CLI 已实现（`itemark` 二进制，仓库门禁为 `just ci`），本仓库用它自举管理自身事项，配置入口为 `itemark.toml`、数据目录为 `itemark/`。crate 尚未发布到 crates.io；预编译包经 GitHub Release 分发，发布由 tag 触发。下列 Markdown 文档承载需求、领域词汇与设计约定。

从源码运行：安装 Rust 工具链后，在仓库内运行 `cargo build --release`，再执行 `target/release/itemark <命令>`；开发时也可用 `cargo run -q -- <命令>`。也可在源码仓库安装到 Cargo 的用户级二进制目录：`cargo install --locked --path .`，之后用 `cargo uninstall itemark` 卸载。该方式只安装 CLI，不安装 skill，也不触碰项目记录。crate 尚未发布到 crates.io，因此目前不能使用 `cargo install itemark`。

## 发布流程

正式发布由 tag 触发：推送形如 `v0.1.0` 的 tag 后，[发布 workflow](.github/workflows/release.yml) 构建全部目标并创建 GitHub Release。tag 中的版本必须与 `Cargo.toml` 一致，否则 workflow 直接失败。手动运行同一 workflow（`workflow_dispatch`，输入版本）只做打包验证，不创建 Release。

```sh
git tag v0.1.0
git push origin v0.1.0
```

### Release 资产

| 资产 | 说明 |
| --- | --- |
| `itemark-v<V>-<target>.tar.gz` | Linux / macOS 二进制归档 |
| `itemark-v<V>-x86_64-pc-windows-msvc.zip` | Windows 二进制归档 |
| `itemark-skill-v<V>.zip` | skill 包，独立于二进制包 |
| `SHA256SUMS` | 以上全部资产的 SHA-256 |

目标为 Linux `x86_64-unknown-linux-gnu` / `aarch64-unknown-linux-gnu`、macOS `x86_64-apple-darwin` / `aarch64-apple-darwin`、Windows `x86_64-pc-windows-msvc`。workflow 会在对应的原生 runner 上实测归档安装/卸载与在线安装/卸载，全部通过才创建 Release。

## 预编译安装

安装器可远程执行，不需要先下载归档；默认安装 latest。安装时会下载对应平台归档与 `SHA256SUMS`，校验和通过才落盘。

```sh
# Unix：默认安装到 ~/.local/bin/itemark
curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.sh | sh

# 固定版本，并同时安装 skill 到 Codex 个人 skill 目录
curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.sh | \
  sh -s -- --version 0.1.0 --skill-profile codex
```

```powershell
# Windows：默认安装到 $LOCALAPPDATA\Programs\Itemark\itemark.exe
irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.ps1 | iex

# 需要传选项时先落盘再执行
irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/install.ps1 -OutFile install.ps1
.\install.ps1 -Version 0.1.0 -SkillProfile codex
```

卸载同样可以远程执行：

```sh
curl -fsSL https://raw.githubusercontent.com/WindLX/itemark/main/scripts/uninstall.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/WindLX/itemark/main/scripts/uninstall.ps1 | iex
```

### 已下载归档时

```sh
# 默认安装到 ~/.local/bin/itemark
./scripts/install.sh --archive /path/to/itemark-v0.1.0-x86_64-unknown-linux-gnu.tar.gz

# 同时安装 skill 包
./scripts/install.sh --archive /path/to/itemark-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  --skill-archive /path/to/itemark-skill-v0.1.0.zip --skill-profile codex
```

Windows 使用 `scripts/install.ps1 -Archive <ZIP路径>`。这条路径同样会校验归档内容；离线安装不校验 `SHA256SUMS`，请先自行核对官方发布页给出的校验和：

```sh
curl -fsSLO https://github.com/WindLX/itemark/releases/download/v0.1.0/SHA256SUMS
sha256sum -c SHA256SUMS
```

### 安装位置与选项

Unix 默认安装到 `$HOME/.local/bin/itemark`，Windows 默认安装到 `$LOCALAPPDATA\Programs\Itemark\itemark.exe`。安装器不会覆盖已有安装，目标已存在时请先卸载。若 `$prefix/bin` 不在 `PATH` 中，安装器会打印需要追加的 `export PATH` 行。

- Unix：`--prefix DIR`、`--version V|latest`、`--base-url URL`、`--repo OWNER/REPO`、`--skill-profile codex|claude`、`--skill-home DIR`；环境变量 `ITEMARK_VERSION`、`ITEMARK_PREFIX`、`ITEMARK_REPO`、`ITEMARK_BASE_URL`。
- Windows：`-Prefix DIR`、`-Version V|latest`、`-BaseUrl URL`、`-Repo OWNER/REPO`、`-SkillProfile codex|claude`、`-SkillHome DIR`；环境变量 `ITEMARK_VERSION`、`ITEMARK_PREFIX`。
- `--base-url` / `-BaseUrl` 用于镜像或测试，必须同时显式指定版本，不能与 `latest` 连用。

skill 包独立于二进制包：Codex profile 默认复制到 `$HOME/.agents/skills/itemark`，Claude profile 默认复制到 `$HOME/.claude/skills/itemark`；可用 `--skill-home` / `-SkillHome` 覆盖 skill 父目录。卸载器根据安装 receipt 只移除安装器放置的二进制和带 Itemark 管理标记的个人 skill，保留项目记录与父目录；校验失败或安装中途出错会回滚，不留残留。

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

将 skill 安装到个人配置时，最简单的方式是用安装器的 skill 选项（`--skill-profile` / `-SkillProfile`，见上文预编译安装），也可以手动把 `.agents/skills/itemark/` 整个目录复制到 Codex 的 `~/.agents/skills/` 或 Claude Code 的 `~/.claude/skills/`。仓内 Claude 入口引用 `.agents` 下的源文件；独立安装时请复制完整 skill 目录，不依赖仓库文件。

```sh
# Codex
mkdir -p ~/.agents/skills
cp -R .agents/skills/itemark ~/.agents/skills/

# Claude Code
mkdir -p ~/.claude/skills
cp -R .agents/skills/itemark ~/.claude/skills/
```

Windows PowerShell 可将 `.agents/skills/itemark` 复制到 `$HOME\.agents\skills\` 或 `$HOME\.claude\skills\`。卸载时删除对应配置目录中的 `itemark` 子目录；仓内源文件不受影响。二进制安装包与 skill 分开打包，skill 包通过安装器的 `--skill-profile` / `-SkillProfile` 选项安装。

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
