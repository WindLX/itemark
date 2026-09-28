# Worklog

This directory is the live record root for this project. `worklog.toml` sets `root = "worklog"`, so item files live in `items/`, kind templates in `templates/`, and saved summaries in `summaries/`. The item Markdown files are the authoritative records; the index, overview, and handoff are rebuildable projections of them.

Read and update records through the CLI, not by hand:

```bash
worklog list                      # every record with kind, group, and status
worklog show WL-0002              # one record by stable ID
worklog search --text <query>     # find records by content
worklog check                     # required fields, sections, references, completion
worklog summary                   # current overview and handoff (read-only)
```

## Record model

- Work, fact, and term records share project-wide IDs and a single group membership. Groups are one-level and may contain mixed kinds; moving a record between groups preserves its ID, and cross-group references point to the same record.
- Kind definitions and Markdown templates are maintained in project configuration/files. The CLI reads and checks them; it does not edit kind definitions. Kinds may declare a name, description, template, simple fields with required/type/enum constraints, and required nonempty Markdown sections. There is no programmable rule engine.
- Generic lifecycle is independent of kind-specific business state: dropping/deprecating preserves the ID and history; restore preserves the existing business state. Work state is `todo`, `in_progress`, `blocked`, or `done`; facts have a separate verification field; terms have no business state.
- A kind opts into completion checks by declaring a completion field and value. When matched, the CLI requires a nonempty completion explanation and either evidence or an explicit unverified explanation. Kinds without that declaration do not receive this check.

## Status and references

Work-state values are `todo` (待办), `in_progress` (进行中), `blocked` (阻塞), and `done` (完成). Facts use their own verification field; terms have no business status. Group and lifecycle are separate from these values.

`parent` and `depends_on` contain stable item IDs. Leave `parent` null and `depends_on` empty when there is no relationship. Each item uses the sections `目标`, `验收`, `当前下一步`, `历史进展`, and `证据`. Keep corrections in history rather than replacing earlier records. Evidence may be shared and linked by multiple records; note its source and check time.

## History

`WL-0001` through `WL-0008` began as hand-written v0 files under `docs/worklog/`. They were taken over into `items/` on 2026-09-28, and the v0 originals were deleted afterwards. The field-by-field mapping is in [v0-adaptation.md](../docs/v0-adaptation.md); the originals remain readable at commit `4c9e733`.
