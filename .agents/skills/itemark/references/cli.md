# Itemark CLI 参考

> Itemark CLI 的完整行为参考。精确可用选项以所安装版本的 `itemark --help` 为准。

Itemark 是本地 Rust + Clap 同步 CLI。每个 kind 由项目配置和 Markdown 模板定义；用户可增加 kind，不由程序固定为 `work`、`fact`、`term` 三类。模板可声明字段及少量约束（必填、基础类型、枚举）和必填正文节；不支持脚本或任意工作流规则。

## 配置、呈现和校验

- 取值顺序：显式 CLI 参数 → 项目 `itemark.toml` → 内置默认值。不预设全局配置或完整环境变量合并层。
- `itemark.toml` 中的 `root` 指定 Itemark 根目录，模板路径相对该 root 解析，不依赖进程当前目录。
- 仓库默认人读界面为简体中文，项目配置可声明 `language = "zh-CN"`。该设置只选择 CLI 固定提示和摘要模板的语言，不改写既有 Markdown 或翻译用户内容；程序不推断正文语言，也不内置翻译服务。`init` 初始选择使用系统 locale；之后由项目配置决定 CLI 固定提示语言。当前接口不提供 `--language` 覆盖参数；命令名、选项名与取值保持稳定。
- 命令、字段、状态机器值、ID 和 JSON 键保持稳定，不翻译。语言切换不改变字段键或必填正文节的匹配规则。自然语言正文由用户或 AI 撰写；遵循项目 `AGENTS.md` 或等效项目约定的语言偏好。
- 所有命令默认输出面向人的文本；可选 JSON 呈现，同一命令行为不变。错误返回非零退出状态。
- 人读文本在终端下带颜色与强调；全局 `--color auto|always|never` 可显式选择，`auto`（默认）只在标准输出是终端且未设置 `NO_COLOR` 时着色。JSON 永不着色，`never`、`NO_COLOR` 以及管道/重定向输出都保持纯文本。着色只影响呈现，不改变字段、取值或退出状态。
- 人读 `list` 先呈现结果数和筛选条件，再按条目显示层级突出的标题与元数据；TTY 下使用加粗/颜色，JSON 保持机器字段不变。无匹配项时返回空结果和当前筛选条件。
- 每次查询都从当前 Markdown 记录构建索引，不使用数据库或持久查询缓存。
- `add` 可写入尚未填完的记录，不增加 `draft` 状态。它检查已提供的值及记录结构；只有非空字段才做类型/枚举校验。
- `check` 报告缺失或为空的必填字段和正文节，并检查 ID、引用及适用的完成条件。普通写入不触发全量检查，但要保住记录可解析、ID 唯一性和引用完整性。
- summary 与 handoff 的状态分类使用稳定原始键（如 `todo`、`in_progress`、`done_unverified`）；说明性标签仍按项目语言呈现。
- 精确的配置键、基础类型集合、模板语法和字段位置仍待定；kind 只由配置定义，不在 CLI 中写死。

## 命令职责

下表说明当前操作职责。精确 flags 和展示语言以安装版本的 `itemark --help` 为准。

| 命令 | 行为 |
| --- | --- |
| `init` | 为新项目创建 `itemark.toml`、`general` group、可编辑的 `work` / `fact` / `term` kind starter 和 Markdown 模板。已有配置不注入 starter，即使使用 `--force`。 |
| `add` | 按指定 kind 添加记录；可暂缺必填内容，稳定 ID 由 Itemark 分配，文件名同时含 ID 与标题摘要。 |
| `show` | 按稳定 ID 查看一条记录。 |
| `list` | 列出记录，可按 kind、状态或 group 过滤；例如工作事项用 `itemark list --status in_progress` 或 `--status todo`。这三个筛选选项可重复。 |
| `search` | 搜索当前记录内容；引用通过稳定 ID 跨 group 指向原记录。 |
| `update` | 定点更新字段或指定正文节；也可移动记录到另一个 group。 |
| `log` | 即时追加一条带日期的进展记录。 |
| `drop` / `restore` | 废弃或恢复任一 kind 的记录；恢复保留废弃前的业务状态。 |
| `group list/add/show` | 查看、新建或查看一级 group；group 可包含不同 kind。 |
| `kind list/show/check` | 查看并检查项目配置中的 kind；不提供逐字段 schema 编辑器。 |
| `check` | 检查记录必填项、正文节、关系及适用的完成条件。 |
| `summary` | 生成当前总览或交接，注明来源 ID 和生成时点；默认只输出，显式指定保存位置后才写文件。`--handoff` 只列进行中、阻塞、完成但未验证三类及其下一步，`--json` 下返回同一交接结构而不是总览结构。 |

## 当前命令形式

所有子命令均可使用 `--project <目录>`、`--root <目录>`、`--color <auto|always|never>` 和 `--json`。`--project` 指定项目目录；未指定时向上查找 `itemark.toml`。全局 `-h/--help` 与 `-V/--version` 用于查看帮助和版本；命令选项以当前帮助为准。

| 命令 | 参数与专属选项 |
| --- | --- |
| `init` | `--force` |
| `add` | `--kind <名称>`、`--group <名称>`（必填）；`--title <文本>`、可重复 `--set <字段=值>`、`--body <文件>` |
| `show` | `<ID>` |
| `list` | 可重复使用的筛选：`--group <名称>`、`--kind <名称>`、`--status <状态>`；`--all` 包含已废弃项 |
| `search` | `<文本>`；`--all` 包含已废弃项 |
| `update` | `<ID>`；可重复 `--set <字段=值>`、`--unset <字段>`、`--section <分节=内容>`、`--append <分节=行>`；`--group <名称>`；`--force` 跳过旧内容比对 |
| `log` | `<ID> <文本>`；`--date <YYYY-MM-DD>`、可重复 `--section <名称=内容>` |
| `drop` | `<ID>`；`--reason <文本>` |
| `restore` | `<ID>` |
| `group list` | 无专属参数 |
| `group add` / `group show` | `<名称>` |
| `kind list` | 无专属参数 |
| `kind show` / `kind check` | 可选 `[kind 名称]` |
| `check` | 可选 `[ID]`；省略时检查所有记录 |
| `summary` | `--at <时间戳>`、`--save <文件>`、`--handoff` |

`update --force` 会跳过写入前旧内容比对，外部修改可能被本次结果覆盖。示例和值定义仍以项目配置与当前 CLI 行为为准。

## 记录行为

记录采用 YAML 头部和 Markdown 正文。`update` 只改指定字段或正文节，保留其余正文；不承诺 YAML 原文顺序或空白的逐字往返。每条记录恰属一个一级 group；group 可混合 kind，不嵌套。移动改变归属但不改变稳定 ID，引用按 ID 工作，不复制被引用内容。

`list` 的 `--group`、`--kind` 和 `--status` 可各自重复指定。同一字段的多个值按 OR 匹配，不同字段之间按 AND 组合；重复的同值只参与一次筛选。逗号是普通字面字符，不会拆成多个值。例如 `itemark list --status todo --status in_progress --kind work` 表示状态为 `todo` 或 `in_progress`，且 kind 为 `work`。此多值筛选只适用于 `list`，不扩展 `search`、`summary` 或按 ID 定位/编辑的参数。

普通写入即校验 `parent`、`depends_on` 指向的 ID 存在且不是自引用：`depends_on` 可用 `[IM-42, IM-43]` 或 `IM-42,IM-43` 的列表写法，`parent` 只接受单个 ID。

这类结构关系按事项 ID 工作。正文内稳定引用的语法及对缺失、废弃或归档目标的检查行为仍在讨论，本参考不将其描述为当前可用功能。

`update` 在写入前比对本次操作开始时读到的原文与磁盘内容；不一致即报冲突并以非零状态退出，不静默覆盖。`--force` 只跳过这一步（直接用本次结果覆盖，外部改动会被丢弃），字段与引用校验、完成条件、ID 与格式校验、项目写锁照常生效。

业务状态由 kind 决定。工作事项当前原始值为 `todo`、`in_progress`、`blocked`、`done`；事实使用 `unverified`、`verified`、`disputed` 核实值，术语不要求业务状态。kind、group、状态和枚举是项目数据，不随界面语言翻译；`--status` 接收配置中的原始值，例如 `itemark list --status in_progress`。自定义 kind 可以声明其他值，不能把工作事项四个状态当作所有 kind 的固定白名单。CLI 标签和提示可本地化。漏传 `--status` 值时会返回本地化用法错误；传入一个当前无记录匹配的值时，CLI 报告空结果和筛选条件，不把它当作非法状态。`--help` 显示工作事项的原始示例值并说明自定义 kind 的有效值来自 kind 声明。`drop` / `restore` 是独立的通用记录生命周期，不能替代业务状态。

kind 可选地声明一个完成字段及完成值（配置键 `completion_field` / `completion_values`）。仅当该映射已声明、且**本次写入**把该字段改成完成值时，CLI 要求非空的统一完成说明，以及证据引用或明确的未验证说明；「明确的未验证说明」需以 `未验证`（或 `unverified`）开头。完成说明与证据优先读头部的 `completion_note` / `completion_evidence`，为空时回退读正文的 `## 完成说明` / `## 证据` 分节。`check` 对所有记录应用同一条件；未声明映射的 kind 不应用此规则。

## 典型操作

1. `init` 建立项目和 starter kinds；用 `itemark add --kind work --group general --title "准备搜索交接" --set status=todo` 新建工作事项，再用 `show` 读回。事实可用 `itemark add --kind fact --group general --title "事实标题" --set verification=unverified` 新建。
2. 用 `group add` 建立一级 group，把同组的工作事项、事实和术语分别加入；用 `list`、`search` 查找，并以 ID 互相引用。
3. 用 `update` 改标题、配置字段或一个正文节，或将记录移动到另一 group；用 `log` 追加进展。未触碰的 Markdown 正文继续保留。
4. 用 `drop` 暂停记录，之后 `restore` 回到原业务状态；用 `check` 查看尚缺的必填信息。
5. 用 `summary` 在 stdout 查看交接；只有明确要求保存时才传入保存位置。用 `--json` 可把同一结果交给脚本读取。

这些场景在 CLI 中已可执行。记录的权威来源是 Markdown 文件；总览和交接由当前文件生成，不构成第二份状态。语言设置只影响 CLI 固定文案，不会更改模板创建后的用户正文。

## 已确认设计（当前尚未实现）

归档与废弃是独立生命周期。计划命令为 `itemark archive <ID>...` 和 `itemark unarchive <ID>...`；归档保留 ID 与业务状态，记录移至唯一来源目录 `<root>/archive`。默认 `list`、`search`、`summary` 隐藏归档项；三者提供 `--include-archived`，`list --archived` 仅显示归档项。`--all` 只控制废弃项，和归档选项互相独立。`show`、`check` 与按 ID 引用仍跨 `items/` 和 `archive/` 工作。

计划的聚合命令为 `itemark merge <ID>... --title <标题> --group <group> [--set FIELD=VALUE ...] [--parent <ID>] [--dry-run]`。参数重复去重后须至少有两个不同 ID；来源必须同 kind，可跨 group；已归档来源可合并，废弃来源须先 `restore`。标题和目标 group 必须明确指定。`--dry-run` 展示候选 ID 与结果，不分配或保留 ID；正式写入在锁内重建索引、重新验证并分配 ID。

聚合字段中，相同值直接保留，只有一个非空值时保留；不同值（包括业务状态或 parent）必须显式解决，禁止静默覆盖。`--set` 只设置 kind 声明字段，parent 使用专门选项。来源归档后记录 `merged_into`，原历史只追加去向，不改写来源历史或代码示例；新项以 `merged_from` 保留来源 ID。普通正文引用、外部 `depends_on` 和 parent 指向新项；聚合项的依赖取来源外部依赖并集、去重并排除来源之间的依赖。重定向不得产生自引用或重复关系。

新聚合项带独立的 `needs_review` 标记。计划 `list --needs-review` 筛选该标记，`check` 对其提示，`update <ID> --reviewed` 清除它；这不改变业务状态，完成值的完成说明与证据规则仍适用。写入前准备所有结果并检查字段与关系冲突；若未先解决则不开始写入。此操作不承诺跨文件事务原子性：I/O 错误返回非零并说明已完成/未完成范围，可做简单回滚，不引入事务日志框架。以上命令和行为是已确认设计，尚待实现；安装版本的可用接口仍以实际 `itemark --help` 为准。

建议的新项目布局为 `itemark.toml`、`<root>/items/`、`<root>/templates/` 和 `<root>/summaries/`。这是布局建议，不要求其他项目自动迁移既有手工记录。
