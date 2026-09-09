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
