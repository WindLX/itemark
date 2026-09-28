# Worklog

This directory is the hand-maintained source of truth for records. Each record has one Markdown file and a project-wide stable ID (`WL-0001`, `WL-0002`, ...); its filename and group may change without changing its ID. The index and handoff below are manually maintained summaries until the CLI can regenerate them. The CLI is not implemented yet.

## Record model

- Work, fact, and term records share project-wide IDs and a single group membership. Groups are one-level and may contain mixed kinds; moving a record between groups preserves its ID, and cross-group references point to the same record.
- Kind definitions and Markdown templates are maintained in project configuration/files. The CLI reads and checks them; it does not edit kind definitions. Kinds may declare a name, description, template, simple fields with required/type/enum constraints, and required nonempty Markdown sections. There is no programmable rule engine.
- Generic lifecycle is independent of kind-specific business state: dropping/deprecating preserves the ID and history; restore preserves the existing business state. Work state is `todo`, `in_progress`, `blocked`, or `done`; facts have a separate verification field; terms have no business state.
- A kind opts into completion checks by declaring a completion field and value. When matched, the CLI requires a nonempty completion explanation and either evidence or an explicit unverified explanation. Kinds without that declaration do not receive this check.
- Current item files use the v0 hand-maintained shape. They are awaiting CLI adaptation and are not examples of the final accepted format. WL-0001 remains unchanged.

## Status and references

Work-state values are `todo` (待办), `in_progress` (进行中), `blocked` (阻塞), and `done` (完成). Facts use their own verification field; terms have no business status. Group and lifecycle are separate from these values.

`parent` and `depends_on` contain stable item IDs. Leave `parent` null and `depends_on` empty when there is no relationship. Each item uses the sections `目标`, `验收`, `当前下一步`, `历史进展`, and `证据`. Keep corrections in history rather than replacing earlier records. Evidence may be shared and linked by multiple records; note its source and check time.

## Work items

| ID | State | Title | File | Depends on |
| --- | --- | --- | --- | --- |
| WL-0001 | done | Establish the manual worklog baseline | [WL-0001.md](WL-0001.md) | — |
| WL-0002 | todo | Build the minimum worklog CLI | [WL-0002.md](WL-0002.md) | — |
| WL-0003 | todo | Verify v0 takeover and cross-session continuation | [WL-0003.md](WL-0003.md) | WL-0004, WL-0008 |
| WL-0004 | todo | Verify concurrent CLI writes and stale-update conflicts | [WL-0004.md](WL-0004.md) | WL-0007 |
| WL-0005 | todo | Manage groups and query records | [WL-0005.md](WL-0005.md) | WL-0002 |
| WL-0006 | todo | Update, move, and log record changes | [WL-0006.md](WL-0006.md) | WL-0005 |
| WL-0007 | todo | Drop, restore, and validate records | [WL-0007.md](WL-0007.md) | WL-0006 |
| WL-0008 | todo | Build overview and handoff views | [WL-0008.md](WL-0008.md) | WL-0007 |

## Handoff

**Maintained:** 2026-09-28 (Asia/Shanghai), by hand.

**Current state:** WL-0001 records the completed manual bootstrap. The CLI is not implemented. The authorized implementation plan is split across WL-0002 through WL-0008; no item has `ready-for-agent` status. WL-0002 has no unfinished dependency and is the current frontier.

**Next:** Start WL-0002. It includes first-write project locking and unique ID allocation, so concurrency protection is present from the first write path. WL-0004 later exercises the agreed races using real CLI processes.

**Sources:** WL-0001 through WL-0008, as of 2026-09-28. The v0 manual records remain awaiting CLI adaptation; they are not final-format examples. Current kinds, grouping, lifecycle, completion checks, and task split reflect user-approved decisions relayed in this conversation.
