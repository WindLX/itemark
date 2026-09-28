# 配置与记录示例

> 这些片段用于审查记录形状，不表示 CLI 或解析器已经实现。TOML 表名、字段名和模板插值形式均为草案；示例值仅展示已确认的能力，不定义通用规则语言。

## 项目 TOML 草案

```toml
# Worklog 的项目级配置。CLI 参数覆盖它；它覆盖内置默认值。
language = "zh-CN" # 人读固定提示/摘要模板的语言；键位置仍为工程草案。
root = "worklog" # 相对本 worklog.toml 所在目录；root 键是工程草案。

[[kinds]]
name = "work"
description = "可继续推进的工作事项"
template = "templates/work.md" # 相对 Worklog root，不相对运行时 CWD。
required_sections = ["目标"]
completion_field = "status"
completion_values = ["done"]
fields = [
  { name = "title", type = "string", required = true },
  { name = "status", type = "enum", required = true, values = ["todo", "in_progress", "blocked", "done"] },
  { name = "owner", type = "string", required = true },
]

[[kinds]]
name = "fact"
description = "有可定位来源支持的事实记录"
template = "templates/fact.md"
required_sections = ["陈述", "来源"]
fields = [{ name = "title", type = "string", required = true }]

[[kinds]]
name = "term"
description = "术语定义"
template = "templates/term.md"
required_sections = ["定义"]
fields = [{ name = "title", type = "string", required = true }]

# 用户创建的 group 示例，不是 init 的默认值；group list/add/show 是日常管理接口。
[[groups]]
name = "产品"

[[groups]]
name = "研究"
```

`work`、`fact`、`term` 是用户在配置中创建的示例，不是封闭的内置类型或 `init` 默认值；增加 kind 不需要改 CLI 代码。示例仅使用文本和枚举字段。`completion_field` 与 `completion_values` 是示意字段名；确切 TOML schema 仍待定。

kind 可选声明所需正文节。`add` 可将字段或节留空；它仍创建普通记录，不会引入 `draft` 状态。`check` 报告必填字段缺失或为空、必填节缺失或为空；空值不再做类型/枚举检查，非空值才检查其声明约束。示例的 `owner` 和“目标”节可空时，`check` 会报告它们；实际运行结果未展示。

若 kind 声明完成字段和值（例如工作事项的 `status = "done"`），该值匹配时必须提供非空的统一完成说明，以及证据引用或明确的未验证说明。没有该映射的 kind 不应用完成条件。上述完成说明和证据键的精确字段名尚未确认。

Group 示例表示用户创建了两个一级 group；它们不是预设默认值。Group 可混合 kind，每条记录仅属于一个 group。`group` 命令是日常管理接口；元数据的确切持久化位置尚未冻结。

## 本地 Markdown 模板示例

推荐相对 `worklog.toml` 的路径为 `worklog/templates/work.md`。模板文件只放 Markdown 和待填位置，不放脚本或条件规则；`{{field}}` 仅表示待定的字段插值示意。

```markdown
---
id: "{{id}}"
kind: "work"
group: "{{group}}"
title: "{{title}}"
status: "todo"
owner: ""
---
## 目标

## 进展
```

## YAML 头部与 Markdown 正文示例

下面是一条可创建但尚未满足 kind 必填条件的工作事项形状。建议路径为 `<root>/items/WL-0123.md`。它只用于审阅格式，不是由 CLI 写出的结果：

```markdown
---
id: WL-0123
kind: work
group: 产品
title: 准备搜索交接
status: in_progress
owner: ""
---
## 目标

## 进展
- 2026-09-28：需要补充目标说明。

## 补充说明
保留的 Markdown 正文示例。
```

按上面的 kind 示例，`owner` 与必填“目标”节为空，因此全量 `check` 应报告这两项；普通 `add` 可以保存此未填完记录。`update` 应只改指定字段或正文节，未指定的正文（例如“补充说明”）继续保留。此处的 front matter 字段名仅是 schema 样例，不声明字段位置已冻结。建议布局不要求其他项目自动迁移既有手工记录。
