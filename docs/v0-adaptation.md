# v0 记录适配说明（WL-0001 … WL-0008）

本文档记录八个 v0 记录的形状、到新格式的逐字段映射，以及接管的实际执行结果。第 1–3 节是接管前的源码推导，第 4.0 节是已实测的接管事实。

- 事实来源（源码）：`src/record/mod.rs`、`src/record/markdown.rs`、`src/record/index.rs`、`src/domain/front_matter.rs`、`src/domain/id.rs`、`src/domain/scalar.rs`、`src/kind.rs`、`src/workspace/config.rs`、`src/workspace/mod.rs`、`src/checks.rs`、`src/cli/item.rs`、`src/cli/args.rs`、`src/cli/mod.rs`、`src/cli/config_cmd.rs`。
- 事实来源（记录）：八个 v0 原文件曾在 `docs/worklog/WL-0001.md` … `WL-0008.md`；接管完成后已从工作区删除，仍可在提交 `4c9e733` 中读到。接管后的副本在 `worklog/items/WL-0001.md` … `WL-0008.md`。
- 接管状态：仓库已有 `worklog.toml` 与 `worklog/` 目录，八个记录已被 CLI 收录与校验（详见第 4.0 节）。

---

## 1. 已确认的新记录格式契约

### 1.1 头部键序与规范键

`add` 写出的头部字段顺序由 `src/cli/item.rs` 的两个常量加上 kind 声明字段决定，原文为：

```rust
/// 头部规范键序；kind 声明字段插在 `title` 之后，未知键保留在末尾。
const HEAD_KEYS: [&str; 4] = ["id", "kind", "group", "title"];
const TAIL_KEYS: [&str; 4] = [
    "parent",
    "depends_on",
    "completion_note",
    "completion_evidence",
];
```

```rust
/// 除 kind 声明字段外，`--set` 还允许写入的规范键。
///
/// `id` 由工具分配，`kind`/`group` 有专用参数，都不走 `--set`。
const CANONICAL_SET_KEYS: [&str; 4] = [
    "parent",
    "depends_on",
    "completion_note",
    "completion_evidence",
];
```

据此，一条经 `worklog add` 新建的记录头部顺序为：

1. `id`
2. `kind`
3. `group`
4. `title`
5. kind 声明的字段（按 kind 配置中的声明顺序，`src/cli/item.rs:269-271`）
6. `parent`
7. `depends_on`
8. `completion_note`
9. `completion_evidence`
10. 模板中其余未识别字段与未知键（保持在末尾，`src/cli/item.rs:277-286`）

补充事实：

- `TAIL_KEYS` 中的键只在「模板里出现过」或「显式传入非空值」时才写出，不会主动补空，见 `src/cli/item.rs:272-276` 及其注释「模板未出现的尾部键（parent/depends_on/完成依据）不主动补空，避免污染记录」。
- 上述重排只发生在 `add` 的 `ordered_fields`；`update`/`log`/`drop`/`restore` 通过 `Record::set` 原地改值或追加到末尾，不整体重排，见 `src/record/mod.rs:166-171`。
- `--set` 只接受 kind 声明字段与 `CANONICAL_SET_KEYS`；`id`、`kind`、`group` 不通过 `--set` 写入，见 `src/cli/item.rs:505-525` 与 `src/cli/args.rs:99-101`、`146-162`。

### 1.2 CLI 读取/写入的键

| 键 | 读写位置与行为 |
| --- | --- |
| `id` | `Record::id()` 缺失即报错「记录缺少 `id` 字段」（`src/record/mod.rs:57-60`）；由 `allocate_id` 分配（`src/workspace/mod.rs:134-137`）。 |
| `kind` | `Record::kind()` 缺失即报错「记录缺少 `kind` 字段」（`src/record/mod.rs:62-65`）；`add` 由 `--kind` 写入（`src/cli/args.rs:86-89`）。 |
| `group` | `Record::group()` 缺失返回空串（`src/record/mod.rs:67-69`）；`add` 要求 `--group` 且必须是已声明 group（`src/cli/args.rs:91-94`，`src/cli/item.rs:42-48`）。 |
| `title` | `Record::title()` 缺失返回空串（`src/record/mod.rs:71-73`）；`add` 可用 `--title` 或 `--set title=…`（`src/cli/item.rs:62-66`）。 |
| kind 声明字段 | 字段定义在项目配置 `kinds[].fields`（`src/workspace/config.rs:23-33`、`49-51`），取值按 `type`（`string`/`date`/`int`/`bool`/`enum`）校验（`src/kind.rs:77-107`）。 |
| `parent` | 单值引用；`null`、空串或缺失都视为无父项（`src/record/mod.rs:101-106`）。 |
| `depends_on` | 引用列表；单值按一个 ID、列表按多个 ID 读取（`src/record/mod.rs:108-118`、`src/domain/scalar.rs:60-72`）。 |
| `completion_note` | 先取头部该键；为空则回退到正文 `## 完成说明` 分节（`src/record/mod.rs:120-133`，常量 `DEFAULT_COMPLETION_SECTION = "完成说明"`，`src/record/mod.rs:19`）。 |
| `completion_evidence` | 先取头部该键；为空则回退到正文 `## 证据` 分节（`src/record/mod.rs:135-148`，常量 `DEFAULT_EVIDENCE_SECTION = "证据"`，`src/record/mod.rs:20`）。 |
| `dropped` | 生命周期键；`drop` 写 `dropped: true`，`restore` 写 `dropped: false`（`src/cli/item.rs:214-229`）。读取时 `Bool(true)` 或文本 `"true"` 视为已废弃，缺失视为有效（`src/record/mod.rs:83-91`）。该键不在 `HEAD_KEYS`/`TAIL_KEYS` 中，因此新建记录默认不写。 |
| 未知键 | 头部解析与渲染保留文件顺序与未知键，包括未识别的键（`src/domain/front_matter.rs:16-48`；测试 `unknown_keys_survive_a_round_trip`，`src/domain/front_matter.rs:68-72`）。 |

头部只接受扁平形态：标量或标量列表；嵌套映射、锚点与自定义标签会被拒绝（`src/domain/front_matter.rs:26-35`、`src/domain/scalar.rs:89-107`）。

### 1.3 正文分节规则

- 正文由「前言」和若干 **二级标题（H2）** 分节组成；`##` 边界用 pulldown-cmark 解析，H1 及其它内容留在前言中并原样保留（`src/record/markdown.rs:1-5`、`30-89`）。
- 分节渲染统一为 `## <标题>`，前言若非空则保留在最前（`src/record/markdown.rs:91-110`）。
- 必填分节由 kind 的 `required_sections` 声明：分节必须存在且分节正文非空白，否则 `check` 报 `missing_section`（`src/record/markdown.rs:167-178`、`src/checks.rs:230-236`）。
- 完成说明/证据有正文回退：头部键为空时分别回退到 `## 完成说明` 与 `## 证据`（`src/record/mod.rs:120-148`）。

### 1.4 ID 规则

- 前缀常量：`pub const ID_PREFIX: &str = "WL-";`（`src/domain/id.rs:6`）。
- 渲染固定 4 位零填充：`format!("{ID_PREFIX}{number:04}")`（`src/domain/id.rs:20-22`），因此 `format_id(1) == "WL-0001"`；序号超过 4 位时自然溢出位数（如 `WL-12345`）。
- 解析要求前缀后全为 ASCII 数字（`src/domain/id.rs:9-16`）。
- 分配规则为「当前最大序号 + 1」：

```rust
/// 分配一个项目范围内未使用的最小 ID。
pub fn next_id(&self) -> Result<String> {
    let mut max = 0u64;
    for record in &self.records {
        if let Some(number) = id_number(record.id()?) {
            max = max.max(number);
        }
    }
    Ok(format_id(max + 1))
}
```

（`src/record/index.rs:59-68`；注意函数注释写「最小 ID」，实现实为最大序号 + 1。`Transaction::allocate_id` 调用它，`src/workspace/mod.rs:134-137`；`create` 对已存在 ID 明确失败，不覆盖，`src/workspace/mod.rs:140-155`。）

### 1.5 业务状态与完成检查

- 业务状态字段名必须是 `status`：`KindConfig::status_field()` 只查找名为 `status` 的字段（`src/workspace/config.rs:53-61`）。kind 未声明时，记录没有业务状态（`src/record/mod.rs:95-99`）。
- `WORK_STATUSES` 只是领域常量 `["todo", "in_progress", "blocked", "done"]`（`src/domain/id.rs:69`），全仓除 `src/domain/mod.rs:7` 的再导出外没有引用；`check` 实际校验的是 kind 字段自身声明的 `enum` 取值（`src/checks.rs:205-228`、`src/kind.rs:95-101`）。
- 固定完成检查仅在 kind 同时声明 `completion_field` 与 `completion_values`、且记录该字段等于某个完成值时才触发；否则不检查（`src/checks.rs:259-292`，`src/workspace/config.rs:63-68`）。
- 触发时要求：非空完成说明，且（证据非空 **或** 完成说明标注「未验证」/小写 `unverified`）；写作时同样强制，`add`/`update` 会直接报错（`src/checks.rs:294-334`；`src/cli/item.rs:91-97`、`153-159`）。

---

## 2. v0 → 新格式逐字段映射表

八个 v0 文件的头部实际只使用五个键：`id`、`title`、`status`、`parent`、`depends_on`（详见第 3 节）。映射如下：

| v0 键 | 新格式对应 | 关系语义与需注意之处 |
| --- | --- | --- |
| `id` | `id` | 同名同义。新格式要求 `WL-` + 数字（4 位零填充），v0 的 `WL-0001`…`WL-0008` 已符合（`src/domain/id.rs:9-22`）。ID 与文件路径无关，是项目范围内唯一稳定标识。 |
| `title` | `title` | 同名同义，位于 `HEAD_KEYS` 第 4 位（`src/cli/item.rs:18`）。新格式的标题取自头部 `title`，不取自正文 H1（`src/record/mod.rs:71-73`）。 |
| `status` | kind 声明的业务状态字段 | `status` 不是通用规范键，而是 kind 声明的字段：只有 kind 配置里声明了名为 `status` 的字段时，`Record::status()` 才返回它（`src/workspace/config.rs:53-61`）。保留原值的前提是该字段的 `type`/`values` 允许 `todo`/`done` 等取值，否则 `check` 报 `field_value`（`src/checks.rs:205-228`）。 |
| `parent` | `parent` | 父子关系，与 `depends_on` 依赖关系相互独立。`parent: null` 在新格式下解析为「无父项」（`display()` 为空串后按无父项处理，`src/record/mod.rs:101-106`），语义不变。 |
| `depends_on` | `depends_on` | 依赖前置项。流式列表 `[WL-0004, WL-0008]` 与块式列表都按多个 ID 解析（`src/domain/front_matter.rs:74-86`、`src/domain/scalar.rs:60-72`）。空列表 `[]` 忽略。引用目标必须存在于同一索引，否则 `check` 报 `broken_reference`（`src/checks.rs:238-254`）。 |
| *（v0 缺失）* `kind` | 新增的必填键 | 新格式要求每条记录有 `kind`；CLI 读取缺失 `kind` 的记录会直接报错（`src/record/mod.rs:62-65`），`check` 报 `missing_field`（`src/checks.rs:141-148`）。 |
| *（v0 缺失）* `group` | 新增的必填键 | 每条记录属于恰好一个一级 group；`add` 强制 `--group` 且必须是已声明 group（`src/cli/args.rs:91-94`、`src/cli/item.rs:42-48`）。`check` 对空 `group` 报 `missing_field`，对未声明的 group 报 `unknown_group`（`src/checks.rs:173-186`）。 |

v0 文件完全没有 `completion_note`、`completion_evidence`、`dropped` 三个键；它们在新格式中均为可选，不是 v0 已有字段的重命名。唯一与完成依据相关的内容是正文的 `## 证据` 分节，CLI 会把它作为 `completion_evidence` 的正文回退读取（`src/record/mod.rs:135-148`）；八个文件都没有 `## 完成说明` 分节。

---

## 3. 八个 v0 文件的逐一处置

### 3.1 共同事实

- 八个文件位于 `docs/worklog/`，而 CLI 索引的目录是 `<root>/items/`（`Config::items_dir()`，`src/workspace/config.rs:183-186`；默认 `root = "worklog"`，`src/workspace/config.rs:15`）。`ItemIndex::scan` 只递归扫描该目录（`src/record/index.rs:17-29`、`134-147`）。
- 因此按当前默认配置，这八个文件不在 CLI 的索引范围内；要进入索引，文件必须位于 `<root>/items/` 之下（或另行覆盖 `root`）。
- 八个文件的头部都只有 `id`、`title`、`status`、`parent`、`depends_on`，都**没有** `kind` 和 `group`。
- 八个文件的正文分节都是 `## 目标`、`## 验收`、`## 当前下一步`、`## 历史进展`、`## 证据`。
- 结论：没有任何一个文件能在**零内容改动**下满足新记录格式契约；`WL-0001` 根本不参与适配，必须保持逐字节不变。

### 3.2 逐文件处置

| 文件 | 现存头部 | 处置 |
| --- | --- | --- |
| `docs/worklog/WL-0001.md` | `id: WL-0001`、`title: Establish the manual worklog baseline`、`status: done`、`parent: null`、`depends_on: []` | **不得改动。** 作为历史兼容样本（grandfathered historical sample），必须逐字节保持不变，即使按新契约它缺少 `kind`/`group`、无法通过 `check`，也不得补写、移动或重写。 |
| `docs/worklog/WL-0002.md` | `status: todo`、`parent: null`、`depends_on: []` | 需要新增 `kind`、新增 `group`；`status` 仅在接管的 kind 声明了 `status` 字段时才保留为业务状态，且取值须被该字段允许。无 `completion_note`/`completion_evidence`。 |
| `docs/worklog/WL-0003.md` | `status: todo`、`parent: null`、`depends_on: [WL-0004, WL-0008]` | 需要新增 `kind`、新增 `group`。正文 H1 为「Check continuation and handoff summaries」，与头部 `title`「Verify v0 takeover and cross-session continuation」不一致；新格式标题只取头部 `title`，H1 留在前言中，因此不构成契约问题，也不要求改名。依赖 `WL-0004`、`WL-0008`。 |
| `docs/worklog/WL-0004.md` | `status: todo`、`parent: null`、`depends_on: [WL-0007]` | 需要新增 `kind`、新增 `group`。正文 H1 为「Prevent lost records across concurrent CLI writes」，与头部 `title` 不同，同上不构成契约问题。依赖 `WL-0007`。 |
| `docs/worklog/WL-0005.md` | `status: todo`、`parent: null`、`depends_on: [WL-0002]` | 需要新增 `kind`、新增 `group`。依赖 `WL-0002`。 |
| `docs/worklog/WL-0006.md` | `status: todo`、`parent: null`、`depends_on: [WL-0005]` | 需要新增 `kind`、新增 `group`。依赖 `WL-0005`。 |
| `docs/worklog/WL-0007.md` | `status: todo`、`parent: null`、`depends_on: [WL-0006]` | 需要新增 `kind`、新增 `group`。依赖 `WL-0006`。 |
| `docs/worklog/WL-0008.md` | `status: todo`、`parent: null`、`depends_on: [WL-0007]` | 需要新增 `kind`、新增 `group`。依赖 `WL-0007`。 |

关于「完成元数据重命名」：

- v0 文件中不存在 `completion_note` / `completion_evidence` 头部键，也没有 `## 完成说明` 分节，因此没有可「重命名」的现成完成元数据。
- 八个文件的 `## 证据` 分节会被 CLI 当作 `completion_evidence` 的正文回退读取（`src/record/mod.rs:135-148`），无需改名即已被识别。
- `WL-0001` 的 `status: done` 若被某个声明了完成字段的 kind 接管，按完成检查规则需要非空完成说明；但 `WL-0001` 属于不得改动的样本，这一冲突只能通过「不接管」来回避。

关于依赖完整性：`WL-0002`…`WL-0008` 之间的依赖边为
`WL-0003 → WL-0004, WL-0008`、`WL-0004 → WL-0007`、`WL-0005 → WL-0002`、`WL-0006 → WL-0005`、`WL-0007 → WL-0006`、`WL-0008 → WL-0007`。这些目标都在八个文件之内；若只接管其中一部分，缺失的目标会被 `check` 报为 `broken_reference`（`src/checks.rs:238-254`）。

---

## 4. 接管执行记录

### 4.0 已执行的接管（2026-09-28，已实测）

本仓库已完成自举，下列内容为事实，不再是待办：

- 新建 `worklog.toml`：`language = "zh-CN"`、`root = "worklog"`；groups 为 `基线`、`实现`、`验证`；一个 kind `work`（`template = "templates/work.md"`、`required_sections = ["目标"]`、`completion_field = "status"`、`completion_values = ["done"]`，字段 `title`(string,required) 与 `status`(enum,required,todo/in_progress/blocked/done)）。
- 新建 `worklog/templates/work.md`（front matter 占位符 + `目标`/`验收`/`当前下一步`/`历史进展`/`证据` 分节）与 `worklog/summaries/`。
- 八个记录已按第 2 节的映射复制为 `worklog/items/WL-0001.md` … `WL-0008.md`：新 front matter 为 `id`、`kind: work`、`group`、`title`、`status`、`parent`、`depends_on`，正文逐字节搬运；group 分配为 WL-0001→`基线`、WL-0003/WL-0004→`验证`、其余→`实现`。
- 第 3.2 节所述「`WL-0001` 处于 `status: done` 却没有完成说明」的冲突，通过在**迁移副本**末尾追加 `## 完成说明` 解决；`docs/worklog/` 下的原始 v0 文件在接管核对无误后已按用户决定删除（内容仍可在提交 `4c9e733` 中读到）。
- 实测：`worklog check` 报「检查记录数：8 / 检查通过」且退出 0；`worklog list`、`worklog summary` 正常列出八条并显示 group。
- ID 分配后果已确认：索引内最大序号为 8，下一条新记录的 ID 为 `WL-0009`。
- 原 `docs/worklog/README.md` 已迁移为 `worklog/README.md`，改写为活跃记录根的说明；`docs/worklog/` 目录已删除。
- 本目录不再更新；后续状态与进展一律通过 CLI 写入 `worklog/items/`。

### 4.1 接管前的未验证事项（历史分析，多数已由 4.0 取代）

- **CLI 从未针对 `docs/worklog/` 下这八个真实文件运行过。** 仓库内不存在 `worklog.toml`，也不存在默认 root 目录 `worklog/`；`tests/cli_workflow.rs` 全部在临时项目目录里自行写入 `worklog.toml` 与 `worklog/items/WL-0001.md` 等文件，没有任何测试引用 `docs/worklog/`。
- 这八个文件当前不在 `<root>/items/` 下，因此按默认配置不会被 `ItemIndex::scan` 收录。如何在不改动 `WL-0001` 的前提下让它们进入索引（复制、覆盖 `root`，或其他方式）未经验证。
- 接管所需的 kind 名、group 名、`required_sections`、`status` 字段的类型与枚举值均未确定：仓库没有 `worklog.toml`，没有声明任何 kind 与 group。
- `check` 对这八个文件逐个运行后的完整问题清单未经验证（按源码推导至少会有 `kind`、`group` 相关的 `missing_field`，但未实测）。
- 完成检查对 `WL-0001`（`status: done`）的实际触发结果未经验证，因为接管它的 kind 尚未定义。
- ID 分配后果未经验证：`next_id` 取索引内最大序号 + 1（`src/record/index.rs:59-68`），若八个文件已进入 items 目录，下一个 ID 应为 `WL-0009`；若 items 目录为空，则从 `WL-0001` 重新开始，会与既有 v0 ID 冲突。
- `docs/cli.md:56` 明确写明：新布局是 `worklog.toml`、`<root>/items/`、`<root>/templates/`、`<root>/summaries/`，且「不要求将当前手工 v0 的 `docs/worklog/` 自动迁移」。本文档不构成迁移方案。

### 4.2 接管前的命令序列（历史方案，实际执行见 4.0）

前置：CLI 二进制用 `cargo build` 构建；仓库门禁为 `just ci`（等价 `just`，配方为 `fmt-check → check → lint → test`，见 `justfile`）。

```bash
# 1. 构建与门禁
cargo build
just ci

# 2. 初始化项目配置与推荐目录
#    init 在当前目录写 worklog.toml（language = "zh-CN"，root = "worklog"），
#    并创建 worklog/{items,templates,summaries}
worklog init

# 3. 在 worklog.toml 中手工声明 groups 与 kinds
#    init 不写入 kind/group 种子数据（src/cli/config_cmd.rs:42-84）

# 4. 按稳定 ID 查看既有记录
worklog show WL-0001

# 5. 逐条接管（每次分配一个新的稳定 ID：当前最大序号 + 1）
worklog add --kind <KIND> --group <GROUP> --title "<标题>" --set status=<值>

# 6. 校验
worklog check
worklog check WL-0001

# 7. 生成总览与交接摘要
worklog summary
worklog summary --handoff
worklog summary --save worklog/summaries/handoff.md
```

命令与参数依据：`worklog init`（`src/cli/args.rs:74-83`）、`worklog add --kind --group --title --set`（`src/cli/args.rs:85-106`）、`worklog show <ID>`（`src/cli/args.rs:108-113`）、`worklog check [ID]`（`src/cli/args.rs:273-278`）、`worklog summary [--handoff] [--save]`（`src/cli/args.rs:280-293`）。
