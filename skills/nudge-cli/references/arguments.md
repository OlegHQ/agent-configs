# Multiline text and structured arguments

Use this reference when text has newlines, quotes, or shell metacharacters, or when a command needs a JSON `--input` file. Plain flags handle everything else; there is no indexed-flag or JSON-envelope scheme to learn.

## Multi-line Markdown: use a file, not shell escaping

Commands that accept long text offer a `--FIELD-file PATH` flag alongside the inline `--FIELD` flag: `--description-file` (issue/project/template/milestone/view), `--content-file` (document), `--body-file` (comment, project status update, agent session report/respond). Every one of these also accepts `-` to read from stdin. Prefer the file form for anything with a newline — it avoids shell-escaping Markdown entirely.

### POSIX shell

```sh
cat > /tmp/runbook.md <<'NUDGE_MARKDOWN'
# Recovery — 世界

Keep "double quotes", 'single quotes', `inline code`, and $variables literally.
NUDGE_MARKDOWN

nudge document update doc_000000000000000000000001 --content-file /tmp/runbook.md
```

The quoted heredoc delimiter (`<<'NUDGE_MARKDOWN'`) prevents expansion of `$variables`, backticks, and backslashes. For an existing file, pass it straight through:

```sh
nudge issue update NUD-7 --description-file ./spec.md
nudge issue comment add NUD-7 --body-file notes.md
cat notes.md | nudge issue comment add NUD-7 --body-file -
```

### PowerShell

PowerShell's own quoting (and 5.1 vs. 7.3+ native argument passing) makes inline multi-line strings fragile; pass a file path instead so PowerShell never has to re-quote the content:

```powershell
$PSNativeCommandArgumentPassing = 'Standard'
nudge document update doc_000000000000000000000001 --content-file .\runbook.md
if ($LASTEXITCODE -ne 0) { throw 'Nudge update failed' }
```

If content must come from a variable instead of a file, write it to a temporary file first rather than interpolating it into the command line, and avoid `Invoke-Expression` to build a Nudge command from workspace text. Native process failures are signaled by `$LASTEXITCODE`; check it after every call in a script.

## Internal entity mentions in Markdown

Issue descriptions, comments, project text, and document `content` are ordinary Markdown. To insert a Notion-style **entity mention** (a selectable chip in the web editor) instead of a plain link, write a relative Markdown link whose fragment is one of the markers below. Unmarked links stay ordinary links.

| Mention | Markdown form |
| --- | --- |
| Issue | `[NUD-7](/issues?issue=NUD-7#nudge-issue)` |
| Project | `[Platform](/projects?project=project_…#nudge-project)` |
| Document / page | `[Runbook](/documents?document=document_…#nudge-document)` |

Rules:

- Use **relative** `/issues`, `/projects`, or `/documents` paths. Absolute `https://…` URLs are never promoted to mentions.
- Issues use the human **identifier** (`NUD-7`) in `?issue=`. Projects and documents use their stable IDs (`project_…`, `document_…`).
- The link text is the cached chip label (identifier, project name, or page title).
- Do **not** use `#nudge-page` / `#nudge-database` for mentions. Those fragments mark **block embeds** (a whole-block page or inline database). A sole-paragraph embed looks like:

```md
[Decisions](/documents?document=document_…#nudge-page)

[Tracking](/documents?document=document_…#nudge-database)
```

Example mid-paragraph mentions in a description file:

```md
Blocked on [NUD-7](/issues?issue=NUD-7#nudge-issue); see [Runbook](/documents?document=document_abc123#nudge-document) in [Platform](/projects?project=project_abc123#nudge-project).
```

```sh
nudge issue update NUD-8 --description-file ./notes.md
nudge document update document_abc123 --content-file ./page.md
```


## Live dashboard widgets in Markdown

Document pages can embed **live metrics** (numbers, progress, radials, item lists, bar/donut charts, tables, creation trends, heatmaps, schedules, and project-dashboard modules). Write a **sole-paragraph** relative link with fragment `#nudge-widget`. The web app evaluates the expression on read; `get_document` / `--raw` still returns the marked link (not the computed number). To read metrics as an agent, use `nudge project dashboard -o json` or `nudge issue list` instead.

### Expression grammar

```
count  issues [in <viewId>] [where <filter> (and <filter>)*] [by status|priority|assignee|project]
count  issues [in <viewId>] [where <filter> (and <filter>)*] by day createdDate [weeks 8|12]
list   issues [in <viewId>] [where <filter> (and <filter>)*] [limit <n>]
list   projects [limit <n>]
ratio  issues [in <viewId>] [where <filterA>…] over issues [in <viewId>] [where <filterB>…]
value  dashboard <path> [focus mine|all] [team <teamId>]
ratio  dashboard <path> [focus mine|all] [team <teamId>]
count  database <documentId> [view <viewId>]
list   database <documentId> [view <viewId>] [limit <n>]
```

Issue count/list queries also accept `scope active|all|backlog|archived` and `sort priority|created|updated|position` after filters. Without explicit scope, the saved view's scope applies, or `active` when no view is supplied. Creation charts usually need `scope all` to include completed work. These options do not apply to the two halves of a ratio expression; use saved views for ratio scopes.

Project modules use:

```
list dashboard attention|focus|projects|review [focus mine|all] [team <teamId>] [sort attention|priority|target|name] [limit 1-50]
```

The default focus is `all`, sort is `attention`, and module limit is 10. Modules reuse the project's decision policy and authorization. Document modules request full dashboard results; the limit controls visible rows within the bounded server response. Use `nudge project dashboard` for full reconciliation. `focus mine` refers to whoever is viewing the document, not its author.

The link's `as=` must match its data shape. An incompatible visualization shows a configuration error instead of a misleading empty chart. Use:

| Expression shape | Default `as` |
|---|---|
| `count` / `value` | `number` |
| `list` / `list projects` | `items` (also `table`; issue/project lists also support `gantt`) |
| `ratio dashboard …` | `radial` |
| `ratio issues … over issues …` | `progress` |
| Grouped issue counts | `bar`, `donut`, or `table` |
| `count … by day createdDate` | `heatmap` or `trend` |
| `list dashboard attention` / `focus` / `review` | matching `attention` / `focus` / `review`; focus also supports `table` |
| `list dashboard projects` | `portfolio` |

### Data set: saved issue views (`in view_…`)

Scope issue metrics to a **saved issue view** with `in <viewId>` (from `nudge view list` / `nudge view create`). The widget reuses that view's **scope, team, project, and filters** — the same filtering engine as the Issues page — then ANDs any extra `where` atoms. Omit `in` for workspace-wide active issues (plus optional `where`). Database widgets still use `view <databaseViewId>` (database schema views, not issue views).

```md
[Open in O1](/widgets?q=count+issues+in+view_da5ce092849348a618a38390&as=number&span=1#nudge-widget)

[P0 in O1](/widgets?q=count+issues+in+view_da5ce092849348a618a38390+where+priority:is:1&as=number&span=1#nudge-widget)

[By status in O1](/widgets?q=count+issues+in+view_da5ce092849348a618a38390+by+status&as=bar&span=1#nudge-widget)
```

Prefer creating a dedicated view for a dashboard (`nudge view create --name "Ops board" --scope active …`), then reference its id in every issue widget so agents and humans share one filter definition.

`<filter>` uses the same atoms as issue saved views: `field:operator:values` with fields `status|assignee|agent|priority|labels|project|relation|dueDate|createdDate|updatedDate` and operators `is|is_not|none|before|after`. Use `assignee:is:me` for the current credential. Priority values are `1` urgent … `4` low. Date presets include `today`, `past`, `future`, `week` (e.g. `dueDate:is:future`). Status/project/assignee values are **ids** (from `nudge status list --team TEAM` / `nudge project list` / members), not display names.

Dashboard `<path>` allowlist: `summary.activeProjectCount`, `summary.attentionProjectCount`, `summary.reviewProjectCount`, `summary.dueSoonCount`, `summary.immediateCandidateCount`, `summary.startedIssueCount`, `summary.weightedCompletion`, `summary.measurableProjectCount`, `summary.weightCoverage`.

**Gantt note:** `as=gantt` draws schedule bars from existing dates only (issues: created→due; projects: start→target). Skip undated rows. Not a full PM Gantt.

**Heatmap note:** default window is **8 weeks** (`weeks 8`); use `weeks 12` only when needed. Cells are compact (contribution-graph style) — keep heatmaps at `span=1` or `span=2`, not full width.

### Layout (`span`)

Optional query param `span=1|1.5|2|3`: one third, half, two thirds, or full width (default). Use three `span=1` KPIs, two `span=1.5` modules, or a `span=1` plus `span=2` row. Widgets retain document order and pack into a responsive grid; ordinary Markdown blocks span a full row and separate widget groups. Narrow containers stack widgets. Height follows content and stretches within each row.

In the editor, use **Add widget** for the library or the **Projects dashboard** slash command for a complete starting layout. Drag the dedicated handle to reorder against widgets or ordinary blocks; use earlier/later buttons for keyboard movement, Settings for width/query, and Duplicate/Remove for iteration. Moves are undoable. Reading a widget link opens the entity; it does not open settings.

Keep widgets as standalone paragraphs with blank lines between them. Preserve normal text, checklists, tables, mentions, and inline database/page blocks when editing a dashboard. Do not represent every note as a widget.

### Markdown form

Put the human label in the link text; put the expression in `q` (spaces as `+`); set viz with `as`; optional `span`:

```md
[Open issues](/widgets?q=count+issues+in+view_abc123&as=number&span=1#nudge-widget)

[Urgent / P0](/widgets?q=count+issues+in+view_abc123+where+priority:is:1&as=number&span=1#nudge-widget)

[P0 share](/widgets?q=ratio+issues+in+view_abc123+where+priority:is:1+over+issues+in+view_abc123&as=progress&span=1#nudge-widget)

[By status](/widgets?q=count+issues+in+view_abc123+by+status&as=bar&span=1#nudge-widget)

[My queue](/widgets?q=list+issues+in+view_abc123+where+assignee:is:me+limit+5&as=items&span=1#nudge-widget)

[Created (8w)](/widgets?q=count+issues+in+view_abc123+by+day+createdDate&as=heatmap&span=1#nudge-widget)

[Portfolio](/widgets?q=ratio+dashboard+summary.weightedCompletion+focus+all&as=radial&span=1#nudge-widget)

[Due schedule](/widgets?q=list+issues+in+view_abc123+where+dueDate:is:future+limit+8&as=gantt&span=2#nudge-widget)

[Projects](/widgets?q=list+projects+limit+8&as=gantt&span=1#nudge-widget)

[Tracking rows](/widgets?q=count+database+document_abc123+view+table&as=number#nudge-widget)
```

Rules:

- Sole paragraph (blank lines around the link) — otherwise it stays an ordinary link.
- Relative `/widgets?…` only. Absolute `https://…` URLs are never promoted.
- Do **not** confuse with `#nudge-issue` / `#nudge-project` / `#nudge-document` (mentions) or `#nudge-page` / `#nudge-database` (page/database embeds).
- Always write via `--content-file` / heredoc (or MCP `content`) so `#` is not treated as a shell comment.

```sh
cat > /tmp/status.md <<'NUDGE_MARKDOWN'
# Weekly status

[Open issues](/widgets?q=count+issues+in+view_abc123&as=number&span=1#nudge-widget)

[Urgent / P0](/widgets?q=count+issues+in+view_abc123+where+priority:is:1&as=number&span=1#nudge-widget)

[P0 share](/widgets?q=ratio+issues+in+view_abc123+where+priority:is:1+over+issues+in+view_abc123&as=progress&span=1#nudge-widget)

[By status](/widgets?q=count+issues+in+view_abc123+by+status&as=bar&span=1#nudge-widget)

[Created (8w)](/widgets?q=count+issues+in+view_abc123+by+day+createdDate&as=heatmap&span=1#nudge-widget)

[Portfolio](/widgets?q=ratio+dashboard+summary.weightedCompletion+focus+all&as=radial&span=1#nudge-widget)

[Due schedule](/widgets?q=list+issues+in+view_abc123+where+dueDate:is:future+limit+8&as=gantt&span=2#nudge-widget)
NUDGE_MARKDOWN

nudge document create --title "Weekly status" --content-file /tmp/status.md
```

## Lists and replace-not-append flags

Repeat a scalar list flag to set the full list: `--label bug --label backend`. This **replaces** the resource's current list — include every value you want kept, not just the new one. Use the resource's `--clear-*` flag (`--clear-labels`, `--clear-tags`, `--clear-related-issues`, `--clear-related-projects`, `--clear-members`, `--clear-priorities`, `--clear-status-ids`) to submit an empty list. Use the literal value `none` on a scalar flag (`--assignee none`, `--milestone none`, `--lead none`) to clear a single nullable field — there is no generic `--unset-*` flag.

## `--input FILE`: when a flag isn't enough

Three families of commands take `--input FILE` (or `--input -` for stdin) for structure a flag can't express. When both `--input` and matching flags are given, flags override the file's top-level keys.

### Document input shape: schema and values

`nudge document create`/`update --input FILE` accepts the same body the API stores for a document. Relevant top-level keys:

```json
{
  "title": "Experiments",
  "content": "",
  "parentId": null,
  "icon": "🧪",
  "tags": ["ops"],
  "relatedIssueIds": [],
  "relatedProjectIds": [],
  "database": {
    "properties": [
      {"id": "name", "name": "Name", "type": "title"},
      {"id": "effort", "name": "Effort", "type": "number"},
      {"id": "status", "name": "Status", "type": "select", "options": [
        {"id": "open", "name": "Open", "color": "blue"}
      ]}
    ],
    "views": [
      {
        "id": "table", "name": "All", "type": "table",
        "propertyIds": ["name", "effort", "status"],
        "filters": [], "filterOperator": "and",
        "sorts": [], "hideEmptyGroups": false
      }
    ]
  }
}
```

A database needs exactly one `title` property; supported property `type`s are `title`, `text`, `number`, `checkbox`, `date`, `url`, `select`, `multi_select`, `formula`, `actor`. Existing properties cannot be removed or change type once created — archive them (`"archived": true` on the property) instead. A record is a child document (`parentId` = the database document's ID) whose `values` array supplies one entry per non-title/non-formula cell:

```json
{
  "title": "Retry study",
  "parentId": "doc_...",
  "values": [
    {"propertyId": "effort", "number": 3},
    {"propertyId": "status", "optionIds": ["open"]}
  ]
}
```

Typed value fields: `text` (string), `number` (float), `checked` (bool, for checkbox), `optionIds` (string array, for select/multi_select), `actorIds` (string array), `date` (`{"start","end","includeTime"}`, date-only or a timed range). The row's `title` field supplies the title cell; title and formula cells cannot be set through `values`. Submitting `values` replaces the record's complete array, so read the current row and keep any cells you are not changing.

### Saved-view filter shape

`nudge view create`/`update --input FILE` sets `filters` and other view fields directly:

```json
{
  "filters": [
    {"id": "f1", "field": "priority", "operator": "in", "values": ["1", "2"]}
  ],
  "displayProperties": ["assignee", "labels"]
}
```

Simple filters (status, priority, labels, assignee, project, team, query, layout, sort, grouping) have dedicated flags (`--status-id`, `--priority`, `--label`, `--assignee`, `--project`, `--team`, `--query`, `--layout`, `--sort`, `--grouping`) — reach for `--input` only for the generic `filters` rule list or `displayProperties`, and prefer flags for anything they cover since flags win when both are supplied.

### Bulk input field names

`nudge issue bulk-create`/`bulk-upsert --input FILE` takes a JSON array of issues, or `{"issues": [...]}`, up to 100 items, using these API field names verbatim:

```
teamId, statusId, title, description, priority, estimate, assignee, labels, parentId, milestoneId, dueAtMs
```

`bulk-upsert` items additionally take `id` — supply it to patch an existing issue, omit it to create one.

`nudge project bulk-upsert --input FILE` takes a JSON array of projects, or `{"projects": [...]}`, up to 100 items, using these API field names verbatim:

```
key, name, description, color, icon, priority, labelIds, leadId, memberIds, startDate, targetDate
```

As with issues, include `id` on an item to patch an existing project (the ID must already exist) and omit it to create one. Both bulk commands validate every item before executing any of them, then run one at a time; use `--dry-run` to validate without writing, and `--rollback-on-error` to delete successful creates if any item in the batch fails (existing-record patches are not rolled back).

### Dynamic action tables

Use the existing dashboard focus policy for actionable tables, rather than a static Do now table:

```text
list dashboard focus projects project_a,project_b focus all assignee user_member readiness ready urgency soon min-score 12 limit 5 as table
```

All clauses are optional. `projects` accepts comma-separated stable project IDs (up to 100). `assignee` accepts a user or service-account ID; `focus mine` means the viewer and excludes unassigned work. Use `focus all assignee …` for a fixed human or agent. Readiness is `ready` (default), `blocked`, or `all`. Urgency is `all` (default), `now` (urgent or due today/overdue), or `soon` (now plus due within seven days). `min-score` accepts 0–100.

Encode the expression in the existing standalone widget link with `as=table`. These filters affect focus rows; project selection scopes the dashboard. Tables refresh every 60 seconds while mounted and show owner, estimate, effective deadline, readiness, linked blockers and score details. Completing a blocker makes its dependent eligible on refresh; related links do not block. Missing or inaccessible blockers remain blocked without revealing restricted details.

Urgency tier precedes score. Score is project priority weight ×4 + task priority weight ×2 + unlocking work (capped at 2) + started (1). Projects take turns within each tier. This orders work, not evidence strength or approval probability. Estimates do not silently change priority. The footer sums visible rows only when team scales are comparable; estimates are points or team sizes, not minutes. Unknown estimates stay unknown.

CLI: `nudge project dashboard --project APP --assignee user_member --readiness ready --urgency soon --min-score 12 --timezone Europe/Zagreb`. Check installed help before using new flags. API fallback: GET `/api/v1/projects/dashboard` with repeated `projectId`, `assigneeId`, `focus=all`, `readiness`, `urgency`, `minScore`, `timezone`, and `full=true`.

### Visual hierarchy and customization

Lead human dashboards with a compact grid of critical counts, attention cards, a clearly named work-completion ring, grouped status/owner charts and a timeline; place detailed action tables below. Consecutive standalone widget paragraphs form the grid: `span=1` is one third, `1.5` half, `2` two thirds, `3` full width. Scope all cards deliberately. Completion is tracked work, not readiness for an unrelated decision. Timeline bars represent creation-to-due dates, not hours spent.

Use Add widget for an individual card. The whole-project template is available through the slash menu, not a persistent button on every document. Card settings separate data/filters, display and width; replacing with a preset replaces that card’s query. Counts support number display; progress bars and rings require ratios. Grouped counts support bar/donut/table. Never turn a raw count into a percentage.
