# 配置与记录示例

> CLI 与解析器已实现；starter 部分对应当前 `itemark init` 输出，后续定制配置仍按项目需要编辑。精确操作行为以 [CLI 参考](../.agents/skills/itemark/references/cli.md) 和当前 `itemark --help` 为准。

## `init` 的 starter 配置

```toml
# 新项目运行 itemark init 的简体中文配置示例。
language = "zh-CN"
root = "itemark-records"

[[groups]]
name = "general"

[[kinds]]
name = "work"
description = "可继续推进的工作事项"
template = "templates/work.md" # 相对 Itemark root，不相对运行时 CWD。
required_sections = ["目标", "验收"]
completion_field = "status"
completion_values = ["done"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "status", type = "enum", required = true, values = ["todo", "in_progress", "blocked", "done"] },
  { name = "completion_note", type = "string" },
  { name = "completion_evidence", type = "string" },
]

[[kinds]]
name = "fact"
description = "有可定位来源支持的事实记录"
template = "templates/fact.md"
required_sections = ["事实陈述", "来源"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "verification", type = "enum", required = true, values = ["unverified", "verified", "disputed"] },
]

[[kinds]]
name = "term"
description = "术语定义"
template = "templates/term.md"
required_sections = ["定义"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "aliases", type = "string" },
]

# 用户可在 starter 配置上增加其他一级 group。
[[groups]]
name = "产品"

[[groups]]
name = "研究"
```

新项目 `itemark init` 会写入 `general` group、`work`/`fact`/`term` 三种可编辑 kind，以及对应 Markdown 模板；这些是配置 starter，不是 CLI 内封闭类型。`work` 以 `status = "done"` 触发完成依据检查；`fact` 从 `verification = "unverified"` 开始；`term` 的 `aliases` 默认为空列表。强制章节按 init 采用的语言写入配置。切换项目语言只影响固定 CLI 文案，不翻译已有 kind/章节规则。`init` 仅为新配置写 starter，已有配置不会被注入默认内容，即使使用 `--force`。

kind 可选声明所需正文节。`add` 可将字段或节留空；它仍创建普通记录，不会引入 `draft` 状态。`check` 报告必填字段缺失或为空、必填节缺失或为空；空值不再做类型/枚举检查，非空值才检查其声明约束。

若 kind 声明完成字段和值（例如工作事项的 `status = "done"`），该值匹配时必须提供非空的完成说明，以及证据引用或明确的未验证说明。当前头部字段为 `completion_note` 和 `completion_evidence`；初始化模板会将二者留空，未声明完成映射的 kind 不应用此固定条件。

`general` 是 init 提供的 starter group；示例中的 `产品` 和 `研究` 是用户追加的一级 group。Group 可混合 kind，每条记录仅属于一个 group；`group` 命令用于日常查看和管理。

## 本地 Markdown 模板示例

相对 `itemark.toml` 的路径为 `itemark-records/templates/work.md`。模板文件只放 Markdown 和待填位置，不放脚本或条件规则；`{{field}}` 表示字段插值。

```markdown
---
id: "{{id}}"
kind: work
group: "{{group}}"
title: "{{title}}"
status: todo
completion_note: ""
completion_evidence: ""
---

## 目标

## 验收

## 当前下一步

## 进展

## 历史进展
```

## YAML 头部与 Markdown 正文示例

下面是一条可创建但尚未满足 kind 必填条件的工作事项形状。示例 ID 与路径为 `IM-42`、`<root>/items/IM-42-准备搜索交接.md`；实际存储布局以项目配置和当前 CLI 为准。新 ID 使用 `IM-数字` 且不补零；文件名可随标题变化，路径不是稳定身份或引用接口：

```markdown
---
id: IM-42
kind: work
group: 产品
title: 准备搜索交接
status: todo
completion_note: ""
completion_evidence: ""
---
## 目标

## 验收

## 进展
- 2026-09-28：需要补充目标说明。

## 补充说明
保留的 Markdown 正文示例。
```

按上面的 kind 示例，必填“目标”和“验收”节为空，因此全量 `check` 应报告这两项；普通 `add` 可以保存此未填完记录。`update` 应只改指定字段或正文节，未指定的正文（例如“补充说明”）继续保留。ID 是稳定引用，文件名只是可随标题变化的可读摘要；前置依赖与普通正文引用的语义不能互相替代。
