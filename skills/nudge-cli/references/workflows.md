# Nudge CLI workflows

Replace uppercase placeholders (ISSUE, TEAM, PROJECT, DOCUMENT, SESSION...) with discovered IDs or keys. Every command accepts an ID, key, or name for teams/projects/statuses/actors; `me` resolves to the current credential and `none` clears a nullable field. A repeated list flag (`--label`, `--related-issue`, `--member`...) replaces the resource's full list, so include existing values you want to keep. Read each command's `--help` for its exact flags before adapting an example.

## Next steps: read them, don't guess

Every successful command ends with a "Next steps" footer in table mode:

```
✓ Created issue NUD-42
Issue NUD-42 · Fix retry backoff
  Status    Todo
  Priority  high
  ...

Next steps
  Change status  nudge issue move NUD-42 <STATUS>
  Comment        nudge issue comment add NUD-42 --body ...
  Archive        nudge issue archive NUD-42
```

In JSON mode (`-o json`, or automatically when piped) the same suggestions travel as `"actions":[{"title","command"}]` alongside `"data"`. Read this instead of composing an unrelated follow-up command — it already has the right identifier filled in.

To pull one field out of a JSON result for the next command:

```sh
issue_id=$(nudge issue create --team NUD --title "Fix retries" -o json | jq -r .data.id)
nudge issue comment add "$issue_id" --body "Investigating"
```

## Issues and projects

### List and triage

```sh
nudge issue list --team NUD --limit 20
nudge issue list --assignee me --sort updated
nudge issue list --project PLAT --status "In Progress" -o json
```

`issue list` filters on query, team, project, milestone, status, assignee, and sort (`position` default, `priority`, `updated`, `created`); there is no arbitrary priority/due-date filter, so filter returned records locally when needed. Priorities: `none`, `urgent`, `high`, `medium`, `low` (or `0`-`4`). `--due` accepts `YYYY-MM-DD`, RFC 3339, or Unix ms, and `none` to clear.

### Create, update, move, assign

```sh
nudge issue create --team NUD --title "Fix retries" --status Todo --assignee me --priority high --label bug
nudge issue create --team NUD --title "Spec" --description-file spec.md --project PLAT --due 2026-10-01
nudge issue update NUD-7 --title "Clarify retries" --priority urgent
nudge issue update NUD-7 --assignee none --clear-labels
nudge issue move NUD-7 "In Progress"
nudge issue assign NUD-7 me
nudge issue bulk-move Done NUD-7 NUD-8 NUD-9
```

`issue update` and `issue create` share the same field flags (`--title`, `--description`/`--description-file`, `--priority`, `--label`, `--milestone`, `--parent`, `--estimate`, `--due`, `--project`, `--status`, `--assignee`); omitted flags on `update` keep their current values. `issue move` only changes workflow status; `issue update --project` moves it to another linked project.

### Relations, duplicates, comments, attachments, subscriptions

```sh
nudge issue relation add NUD-7 NUD-9 --kind blocks
nudge issue relation add NUD-7 NUD-9 --kind related
nudge issue relation list NUD-7
nudge issue relation delete NUD-7 RELATION

nudge issue duplicate mark NUD-7 NUD-3
nudge issue duplicate clear NUD-7 Todo
nudge issue duplicate NUD-7

nudge issue comment add NUD-7 --body "Reproduced on main"
nudge issue comment list NUD-7
nudge issue comment update NUD-7 COMMENT --body "Corrected result"
nudge issue comment delete NUD-7 COMMENT

nudge issue attachment add NUD-7 --title Design --link https://figma.com/file/...
nudge issue attachment list NUD-7
nudge issue attachment delete NUD-7 ATTACHMENT

nudge issue subscribe NUD-7
nudge issue subscription NUD-7
nudge issue unsubscribe NUD-7
```

`issue relation add --kind` is `blocks` or `related` (default `related`); the source blocks the target for `blocks`. `issue duplicate mark ISSUE CANONICAL` marks ISSUE as a duplicate; `issue duplicate clear ISSUE STATUS` removes duplicate state and moves it to an ordinary status. `issue comment`/`issue attachment` take `--command-id` for comments and `--expected-updated-at` for comment edits/deletes to protect concurrent writers; omit them and the CLI looks the current value up automatically where noted in `--help`. `issue attachment add --link` must be an absolute HTTPS URL — it creates a link, not an upload.

### Archive, restore, delete

```sh
nudge issue archive NUD-7
nudge issue restore NUD-7
nudge issue delete NUD-7        # asks you to type the issue key, or pass --yes
```

`issue delete` permanently removes the issue, its comments, attachments, and relations; prefer `issue archive` for routine cleanup.

### Projects, milestones, dependencies, status updates

```sh
nudge project list --query platform
nudge project get PLAT
nudge project create --key PLAT --name Platform --lead me --member ana --priority high
nudge project create --key PLAT --name Platform --description-file brief.md --start 2026-01-01 --target 2026-06-30
nudge project update PLAT --name "Platform Team" --priority high
nudge project update PLAT --lead none --clear-members
nudge project update PLAT --status started --target 2026-09-01
```

`project update --status` is `planned`, `started`, `paused`, `completed`, or `canceled` (create has no `--status` flag; new projects start `planned`). `--label` on project create/update takes label IDs from the project label scope (`nudge label list --scope project`), distinct from issue labels.

```sh
nudge project milestone create PLAT --title "Beta launch" --target 2026-11-01
nudge project milestone create PLAT --title "Spec review" --status started --description "Draft due"
nudge project milestone update PLAT MILESTONE --status completed
nudge project milestone update PLAT MILESTONE --target none
nudge project milestone list PLAT
nudge project milestone delete PLAT MILESTONE
```

Milestone status is `planned`, `started`, `completed`, or `canceled`.

```sh
nudge project dependency add PLAT --blocked-by INFRA
nudge project dependency list PLAT
nudge project dependency remove PLAT --blocked-by INFRA
```

`project dependency add PROJECT --blocked-by OTHER` marks PROJECT as blocked by OTHER (OTHER must finish first).

```sh
nudge project status-update create PLAT --title "Week 12" --body "On track for beta" --health on_track
nudge project status-update list PLAT
nudge project status-update get PLAT UPDATE
nudge project status-update edit PLAT UPDATE --body "Slipped a day" --health at_risk
nudge project status-update delete PLAT UPDATE
```

`--health` is `on_track`, `at_risk`, or `off_track`. `status-update edit` reads the current update and merges in only the flags you pass (title/body/health are otherwise replaced together server-side).

```sh
nudge project team link PLAT NUD
nudge project team list PLAT
nudge project team unlink PLAT NUD
```

Archive a project with `nudge project archive PLAT` (also archives its active issues). There is no restore for an archived or deleted project — the API has no way to clear `archivedAtMs` through a patch — so archiving/deleting a project is effectively final; confirm before running either. `nudge project bulk-upsert` is covered under [imports and bulk operations](#imports-and-bulk-operations).

## Documents

```sh
nudge document list
nudge document list --parent doc_000000000000000000000001
nudge document list --tag runbook --updated-after 2026-09-01
nudge document search retries
nudge document search runbook --tag ops
nudge document get doc_000000000000000000000001
nudge document get doc_000000000000000000000001 --raw > page.md
```

`document list` and `document search` both read `GET /documents/page`, the only paginated document listing endpoint (`GET /documents/search` returns a plain unpaginated array and is not used by either command). `--parent` has no server-side query parameter — it filters client-side after each page loads, so combine it with `--all` to search beyond the first page.

```sh
nudge document create --title "Runbook"
nudge document create --title "Sub-page" --parent doc_000000000000000000000001 --content-file page.md
nudge document update doc_000000000000000000000001 --title "Runbook v2"
nudge document update doc_000000000000000000000001 --content-file page.md
nudge document update doc_000000000000000000000001 --clear-tags
```

`--tag`, `--related-issue`, and `--related-project` replace the full set on the document each time; `--clear-tags`, `--clear-related-issues`, `--clear-related-projects` empty a specific one. For a database document's schema/rows shape and the `--input` JSON keys, see [argument handling](arguments.md#document-input-shape-schema-and-values).

```sh
nudge document move doc_000000000000000000000002 --parent doc_000000000000000000000001 --position 0
nudge document move doc_000000000000000000000002 --parent none
nudge document archive doc_000000000000000000000001
nudge document restore doc_000000000000000000000001
nudge document delete doc_000000000000000000000001   # irreversible, deletes nested pages too
```

`document archive`/`document delete` act on the document and every page nested under it; archive is restorable, delete is not. Both accept `--expected-updated-at` (archive/restore) for optimistic concurrency.

```sh
nudge document revision list doc_000000000000000000000001 --limit 5
nudge document revision get doc_000000000000000000000001 rev_000000000000000000000009
nudge document revision restore doc_000000000000000000000001 rev_000000000000000000000009
```

`revision get` has no dedicated single-revision endpoint — it scans the revision pages looking for that ID, so it can be slow on long histories. `revision restore` appends a new revision instead of rewriting history, and looks up the document's current `updatedAtMs` automatically when `--expected-updated-at` is omitted.

## Agents

```sh
nudge agent list
nudge agent list --issue NUD-7
```

`agent list` returns each agent's delegation ID (the `--agent` value for `agent delegate`) alongside its actor ID (used for issue assignment) — these are different values for the same agent.

### Delegate and drive a session

```sh
nudge agent delegate NUD-7 --agent triage-bot --objective "Investigate flaky retries"
nudge agent delegate NUD-7 --agent agt_... --objective-file objective.md --retry-of ses_...
nudge agent session list NUD-7
nudge agent session get SESSION
```

Delegation starts a session tracked through claim, heartbeat, report, and completion; `--revision`/`--idempotency-key` protect concurrent writers and safe retries. `--revision` defaults to the session's current value when omitted, and idempotency keys default to a generated value that is echoed in a Note so a failed write can be retried with the exact same key.

The runner loop, from the authorized runner's own credential:

```sh
nudge agent session pending                          # queued or reclaimable sessions for this credential
nudge agent session claim SESSION                     # claim it; retains the returned revision
nudge agent session heartbeat SESSION                 # renew the lease while working, without advancing revision
nudge agent session report SESSION --kind progress --body "Reproduced the bug"
nudge agent session report SESSION --kind question --body "Which retry policy is intended?"
nudge agent session report SESSION --kind completion --body "Fixed and tested; PR up"
```

`report --kind` is `progress`, `action`, `question`, `completion`, or `error`. A `question` pauses the session for the delegator's answer; `completion` and `error` are terminal. Page `session pending` with `--cursor`, and page a session's activity history with `session get --after SEQUENCE` (use the last returned activity's sequence number).

The delegator answers a pending question or ends the session:

```sh
nudge agent session respond SESSION --body "Use the documented capped backoff"
nudge agent session cancel SESSION --reason "The requested work has been withdrawn"
```

`respond` requeues the session so the runner claims it again. `cancel --reason` is required. To retry a terminated session under a new one: `nudge agent delegate ISSUE --agent AGENT --objective "..." --retry-of TERMINAL_SESSION`.

## Imports and bulk operations

### Linear import

```sh
nudge import linear export.csv --team NUD
nudge import linear export.json --team NUD --status "In Progress" --project PLAT --dry-run
```

`--project` is optional; both CSV and JSON Linear exports are supported. Always run `--dry-run` first to validate the input and the team's workflow/estimate policy before creating anything — without `--dry-run` the command applies immediately.

### Document import

```sh
nudge import documents pages.json
nudge import documents pages.json --apply
```

Without `--apply`, this only previews dependency order offline; pass `--apply` to actually create the pages and database records described in the file.

### Bulk issue and project writes

```sh
nudge issue bulk-create --input issues.json --dry-run
nudge issue bulk-create --input issues.json --rollback-on-error
cat issues.json | nudge issue bulk-create --input -

nudge issue bulk-upsert --input issues.json --dry-run
nudge project bulk-upsert --input projects.json --dry-run
```

Each file holds a JSON array of items, or an object like `{"issues": [...]}` / `{"projects": [...]}`, up to 100 items. Field names are verbatim API names — see [argument handling](arguments.md#bulk-input-field-names) for the exact list per resource. Items are validated first, then executed one by one; inspect every item's result, not just an aggregate success count. `--rollback-on-error` deletes only the successful creates from that run if any item fails — it does not undo patches applied to already-existing records. `bulk-upsert` patches an item that has an `id` field and creates one that omits it.
