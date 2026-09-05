# Nudge CLI workflows

Examples use the schema-generated interface, verified against the bundled MCP catalog and CLI bindings. Replace uppercase IDs and sample versions with discovered values. Mutations below are examples to adapt to an authorized task, not a sequence to execute indiscriminately.

Use ordinary flags for tool inputs. Repeat list flags (`--labels FIRST --labels SECOND`); use `--clear-labels` for an empty list and `--unset-assignee` for null. Omit a flag to preserve an existing patch field. See the [CLI argument conventions](../SKILL.md#arguments) for nested collections and optional file input. `--output json` controls results, not inputs; successful API data is under `structuredContent.data`.

## Access, identity, and discovery

```sh
nudge auth status
nudge tool get_access_context --output json
nudge tool list_actors --limit 50 --output json
nudge tool list_teams --limit 50 --output json
nudge tool list_issue_statuses --team-id TEAM --limit 50 --output json
nudge tool list_issue_labels --limit 50 --output json
```

Use the access context to determine workspace and project restrictions. `list_actors` requires an unscoped credential; human IDs have kind `user`, agent IDs have kind `service_account`. Never substitute a display name for an actor ID. Follow `nextCursor` using `--cursor` with unchanged filters; an empty relationship page can still have continuation.

## Issue triage and collaboration

For “triage my team's open work,” resolve the team and its status categories, page issues with the same filters, fetch details for candidates, then apply only the requested changes. Priorities are `0` none, `1` urgent, `2` high, `3` medium, `4` low. Backlog/unstarted/started are nonterminal; completed/canceled/duplicate are terminal. Decide whether backlog belongs in the requested set. Inspect archive metadata as well as status. `list_issues` supports query, team, project, milestone, status, assignee and sort (`position`, `priority`, `updated`, `created`); it has no arbitrary priority/due-date filter flag, so filter returned records when needed.

```sh
nudge tool list_issues --team-id TEAM --limit 20 --output json
nudge mcp issue get ISSUE
nudge mcp issue update ISSUE --title "Clarify the acceptance criteria"
```

Use these commands for precise changes:

| Command | Effect |
| --- | --- |
| `nudge tool create_issue --team-id TEAM --status-id STATUS --title 'Investigate retry behavior' --assignee ACTOR` | Create assigned work; capture the returned ID. |
| `nudge tool update_issue --id ISSUE --labels LABEL --description 'Reproduce and record the outcome'` | Replace the label set and description; omitted fields stay unchanged. |
| `nudge tool update_issue --id ISSUE --unset-assignee --clear-labels` | Explicitly unassign and clear labels. |
| `nudge tool move_issues --issue-ids ISSUE --status-id STATUS` | Move up to 100 unique issues to a valid team status. |
| `nudge tool move_issue_project --issue-id ISSUE --project-id PROJECT` | Move to a linked project; inspect help for clearing semantics. |
| `nudge tool mark_issue_duplicate --issue-id ISSUE --canonical-issue-id CANONICAL --command-id UNIQUE_COMMAND` | Mark a duplicate using a stable command key. |
| `nudge tool create_comment --issue-id ISSUE --body 'Reproduction attached' --command-id UNIQUE_COMMAND` | Add a comment; exact retries reuse key and body. |
| `nudge tool subscribe_to_issue --issue-id ISSUE` | Subscribe the authenticated human or service account. |
| `nudge tool unsubscribe_from_issue --issue-id ISSUE` | Remove that subscription. |
| `nudge tool update_issue --id ISSUE --parent-id PARENT --milestone-id MILESTONE --estimate 3 --due-at-ms 1798761600000 --priority 2` | Schedule a sub-issue; use the team's estimate policy, a milestone in the issue's project, and the requested date converted to Unix milliseconds. |
| `nudge tool update_issue --id ISSUE --unset-parent-id --unset-milestone-id --unset-estimate --unset-due-at-ms` | Clear hierarchy/planning fields without changing other values. |
| `nudge tool create_relation --issue-id BLOCKER --target-issue-id BLOCKED --kind blocks` | Make the source block the target; `related` is also supported. |
| `nudge tool clear_issue_duplicate --issue-id ISSUE --status-id STATUS --command-id UNIQUE_COMMAND` | Remove duplicate state and select an ordinary status belonging to the issue's team. |
| `nudge tool archive_issue --issue-id ISSUE` | Remove work from active lists while retaining it. |
| `nudge tool update_issue --id ISSUE --unset-archived-at-ms` | Restore an archived issue. |
| `nudge tool delete_issue --id ISSUE --confirm TEAM-123` | Permanently delete only when requested; confirmation is the fetched issue identifier. |

Discover duplicate context with `get_issue_duplicate`, comments with `list_comments`, and subscription state with `get_issue_subscription`. Read current comment revision before editing/deleting; those tool help pages specify the required command/version fields. Label writes prefer IDs; local legacy names are accepted and canonicalized. Historical reads can still contain names.

For an existing comment:

```sh
nudge tool update_comment --issue-id ISSUE --id COMMENT --expected-updated-at-ms 1700000000000 --command-id UNIQUE_EDIT --body "Corrected result"
nudge tool delete_comment --issue-id ISSUE --id COMMENT --expected-updated-at-ms 1700000000000 --command-id UNIQUE_DELETE
```

These are separate edit/delete examples; use a fresh version and command ID for the selected operation. Replace the timestamp with the comment's current `updatedAtMs`. If `auditStatus` is pending, an exact retry repairs the receipt; a read does not.

Attachments use an explicit resource URL flag so it cannot be mistaken for the authenticated API origin:

```sh
nudge mcp issue attachment create ISSUE --title "Reproduction" --resource-url https://files.example.com/reproduction.txt
```

This creates a HTTPS resource link, not an upload or fetch of the remote file. `--url` always selects the Nudge API origin. Use `list_attachments` to resolve attachment IDs before deletion. Relationship creation/deletion requires both issue endpoints to be visible; inspect `create_relation --help` for supported kinds.

Read `list_relations --issue-id ISSUE` before adding or removing an edge. Use `nudge tool delete_relation --issue-id ISSUE --relation-id RELATION` to remove the selected edge, or `nudge tool delete_attachment --issue-id ISSUE --attachment-id ATTACHMENT` to remove the matching attachment. Managed-media attachments can trigger best-effort file cleanup; removing a relationship is separate from deleting an issue.

## Projects, milestones, dependencies, and progress

```sh
nudge tool list_projects --limit 20 --output json
nudge tool get_project --id PROJECT --output json
nudge tool list_project_milestones --project-id PROJECT --output json
nudge tool list_project_dependencies --project-id PROJECT --output json
```

| Command | Effect |
| --- | --- |
| `nudge tool create_project --key PILOT --name 'Pilot rollout' --lead-id ACTOR --member-ids ACTOR --start-date 2026-10-01 --target-date 2026-10-31` | Create a uniquely keyed project with human or agent owners and date-only planning fields. |
| `nudge tool update_project --id PROJECT --description 'Roll out the pilot' --unset-lead-id --clear-member-ids --unset-target-date` | Edit a project and explicitly clear selected ownership/date fields. |
| `nudge tool create_project_milestone --project-id PROJECT --milestone-title Pilot --milestone-status planned` | Create a milestone; valid statuses also include started/completed/canceled. |
| `nudge tool update_project_milestone --project-id PROJECT --id MILESTONE --status started --unset-target-at-ms` | Start the milestone and clear its date, preserving its description. |
| `nudge tool link_project_dependency --blocking-project-id FIRST --blocked-project-id SECOND` | FIRST must precede SECOND. |
| `nudge tool unlink_project_dependency --blocking-project-id FIRST --blocked-project-id SECOND` | Remove that directed edge. |
| `nudge tool save_project_update --project-id PROJECT --title 'Weekly progress' --body 'Pilot ready for review' --health on_track` | Create a progress update. |
| `nudge tool save_project_update --project-id PROJECT --id UPDATE --title 'Weekly progress' --body 'Pilot needs another day' --health at_risk` | Fully replace title/body/health; all three are required. |

Generated human progress-report commands are under `nudge mcp project status-update`; the existing `project update` command retains its older project-edit behavior. Resolve actor IDs before setting project lead/members. Archive is normally preferable to permanent deletion. Delete tools require their documented confirmation value; never derive confirmation from an unrelated name.

For a project kickoff, create the project, link its team using `link_project_team` (see [workspace workflows](workspace.md)), create milestones, then create issues with the linked `teamId`, `projectId`, and intended `milestoneId`. Project labels come from `list_project_labels` and use `labelIds`, distinct from issue `labels`. Read `get_project` for status and planning fields before patching. Fetch `get_project_milestone` or `get_project_update` with both `projectId` and child `id`; list summaries alone may not give the full working context. `delete_project_milestone` and `delete_project_update` also require that matching pair.

To prepare a progress report, read the project's issues, milestones and dependencies, then publish only supported findings with `save_project_update`; health is `on_track`, `at_risk` or `off_track`. `archive_project` takes `id` and also archives active project issues. `delete_project` requires `id` and `confirm` equal to the actual project key. Both can affect multiple records; inspect state after interruption. There is no dedicated project-restore command in the current CLI.

## Documents, database records, hierarchy, and versions

```sh
nudge tool list_documents --limit 20 --output json
nudge tool search_documents --query "Runbook" --limit 10 --output json
nudge tool get_document --id DOCUMENT --output json
nudge tool list_document_revisions --id DOCUMENT --limit 2 --output json
```

Summary search does not return full markdown. Revision pages contain full snapshots and can need smaller limits for large content. Read `updatedAtMs` before a version-checked update.

For “update the runbook,” search by title/content, retrieve the full document, preserve unrelated markdown, and submit content with its current version. For “organize the handbook,” page summaries and use `parentId`/ordering metadata to identify siblings before moving pages. Documents belong to the workspace: `relatedProjectIds`, `relatedIssueIds` and `relatedInitiativeIds` are relations, not ownership. Add tags, icons and relation arrays through `create_document` or `update_document`; arrays replace existing lists, so retain existing relationships unless removal was requested.

```sh
nudge tool create_document --title Runbook --content '# Recovery
Check readiness first.'
nudge tool update_document --id DOCUMENT --content '# Recovery
Updated procedure.' --expected-updated-at-ms 1700000000000
nudge tool move_document --id DOCUMENT --parent-id PARENT --position 0
nudge tool move_document --id DOCUMENT --unset-parent-id --position 0
nudge tool archive_document --id DOCUMENT --expected-updated-at-ms 1700000000000
```

Replace the sample timestamp with the actual current version. Moves change sibling ordering and reject cycles. Archive/delete include descendants and are nontransactional. Use the typed recovery commands in [administration](administration.md) for unarchive or revision restoration.

`nudge tool delete_document --id DOCUMENT` permanently deletes the page and nested pages; inspect the tree and use it only for the requested deletion. For ordinary retirement use `archive_document`; recovery commands are in [administration](administration.md).

A database is a document with a schema; each record is a child document. Create a minimal database with `create_document` using:

```sh
nudge tool create_document --title Experiments \
  --database-properties-id 0=name \
  --database-properties-name 0=Name \
  --database-properties-type 0=title \
  --database-properties-id 1=effort \
  --database-properties-name 1=Effort \
  --database-properties-type 1=number \
  --database-views-id 0=table \
  --database-views-name 0=All \
  --database-views-type 0=table \
  --database-views-property-ids 0=name \
  --database-views-property-ids 0=effort \
  --clear-database-views-filters 0 \
  --database-views-filter-operator 0=and \
  --clear-database-views-sorts 0 \
  --database-views-hide-empty-groups 0=false
```

Then create a child record with `create_document`:

```sh
nudge tool create_document --title 'Retry study' --parent-id DATABASE_DOCUMENT --values-property-id 0=effort --values-number 0=3
```

Update the record through `update_document` with its current version and the complete intended `values` array. Property IDs are stable; use typed fields such as `number`, `text`, `checked`, `optionIds`, `date`, or `actorIds` matching the schema. Do not send an arbitrary cells map. Actor values support humans and agents. Preserve other cells when replacing the array; `--clear-values` clears it.

Supported property types are `title`, `text`, `number`, `checkbox`, `date`, `url`, `select`, `multi_select`, `formula`, and `actor`. Each database needs exactly one title property. The row's `title` supplies that value; title and formula cells cannot be written through `values`. Text/URL properties use `text`; checkbox uses `checked`; select/multi-select use stable `optionIds`; actors use `actorIds`. A date cell uses `--values-property-id 0=due --values-date-start 0=2026-10-31 --values-date-include-time 0=false`; add `--values-date-end 0=2026-11-01` for a range or use an offset-bearing timestamp with `--values-date-include-time 0=true` for a timed value. Do not confuse these date values with issue/milestone Unix millisecond deadlines.

To change a database's columns, option lists, or table/board views, read its full `database`, edit the relevant property/view, then submit the complete replacement `database` with the page's current version. Preserve property/option IDs and unrelated views so existing records retain meaning. Database views belong inside the document schema; issue saved views use separate `create_saved_view`/`update_saved_view` commands. Check `create_document --help` and `update_document --help` for the installed schema's property types and nested view fields. Revision and archive restore use the typed commands in [administration](administration.md).

## Delegated agent work

```sh
nudge tool list_agents --issue-id ISSUE --output json
nudge tool list_agent_sessions --issue-id ISSUE --output json
nudge tool get_agent_session --session-id SESSION --output json
```

Confirm the issue has an eligible human or agent assignee. Delegation alone does not make the current credential the runner. Claim/heartbeat/report require the assigned runner's credential and current lease/version.

`list_agents` returns two identities: `id` selects the delegation target; `serviceAccountId` is the actor ID for issue assignment. To investigate a stalled task, read its sessions and full paginated history before deciding whether to respond, cancel, or start a retry. Agent configuration and runner credentials live in Settings (see [administration](administration.md)).

```sh
nudge tool delegate_agent_session --issue-id ISSUE --agent-id AGENT --objective 'Investigate retries and report evidence' --idempotency-key UNIQUE_DELEGATION
nudge tool claim_agent_session --session-id SESSION --expected-revision 1 --idempotency-key UNIQUE_CLAIM
nudge tool heartbeat_agent_session --session-id SESSION --expected-revision 2
nudge tool report_agent_session --session-id SESSION --expected-revision 2 --idempotency-key UNIQUE_REPORT --kind progress --body 'Reproduction isolated'
nudge tool report_agent_session --session-id SESSION --expected-revision 3 --idempotency-key UNIQUE_QUESTION --kind question --body 'Which retry policy is intended?'
nudge tool respond_agent_session --session-id SESSION --expected-revision 4 --idempotency-key UNIQUE_RESPONSE --body 'Use the documented capped backoff'
nudge tool report_agent_session --session-id SESSION --expected-revision 5 --idempotency-key UNIQUE_COMPLETION --kind completion --body 'Implemented capped backoff; focused checks passed.'
nudge tool cancel_agent_session --session-id SESSION --expected-revision 5 --idempotency-key UNIQUE_CANCEL --body 'The requested work has been withdrawn.'
nudge tool delegate_agent_session --issue-id ISSUE --agent-id AGENT --objective 'Retry with the corrected configuration' --retry-of-session-id TERMINAL_SESSION --idempotency-key UNIQUE_RETRY
```

Revision numbers above illustrate transitions; use actual returned/current revisions. A question waits for input; responding requeues the session, so the runner claims again. Completion/error reports are terminal. Heartbeats renew the lease without advancing revision. Use a unique key per logical mutation and preserve it for an exact retry. Pending history repair is performed by exact mutation retry, not reads. Follow `nextAfterSequence` for history and ordinary `nextCursor` for session/queue lists.

For an authorized runner: call `list_pending_agent_sessions`, claim a queued session (or expired lease), retain its returned revision, renew the lease while working, and report `progress`, `action`, `question`, `completion` or `error` with a fresh key per logical event. `question` pauses work; resume only after a response and a new claim. Stop on terminal state or loss of the lease. Page the queue with `cursor`, then start a fresh scan for newly queued work. For history, use `get_agent_session --session-id SESSION --after-sequence SEQUENCE` until `activityPageComplete` is true. If a mutation returns `historyDurable:false`, retry that exact mutation to repair history.

## Import previews, application, and partial failures

Document imports default to an offline dependency-order preview:

```sh
nudge import-documents pages.json
nudge import-documents pages.json --apply
```

Example `pages.json`:

```json
[{"externalId":"handbook:root","title":"Handbook","content":"Overview"},{"externalId":"handbook:runbook","parentExternalId":"handbook:root","title":"Runbook","content":"Recovery procedure"}]
```

Database schemas and record values use the same document fields shown above. Stable external IDs preserve identity, but repeated imports can append revisions. Adoption of an existing document requires `targetDocumentId` and its fresh `expectedUpdatedAtMs`. Preview establishes ordering, not remote authorization or exactly-once effects. Apply stops on failure; earlier entries may already exist. Inspect IDs and versions before retrying or editing the batch.

For a single source page, run `nudge tool import_document --external-id handbook:root --title Handbook --content Overview`. It creates or replaces by external identity; every call requires title/content. Omitted tags, relations, and source metadata are cleared, while omitted database, values, and icon are retained. For provenance, supply `--source-system`, `--source-url`, `--source-id`, and `--source-hash` as appropriate. Use `update_document` for a partial edit instead of treating import as a patch.

Linear import defaults to applying changes, so explicitly request its authenticated preflight first:

```sh
nudge import-linear export.csv TEAM STATUS PROJECT --dry-run
nudge import-linear export.csv TEAM STATUS PROJECT
```

`PROJECT` is optional; CSV and JSON exports are supported. Preflight validates input hierarchy and the selected team's workflow/estimate policy without writes; it requires API access. Application creates issues and then parent links. A failure can leave already-created issues or incomplete links. Review the reported mappings and current workspace state before rerunning; do not assume automatic rollback or deduplication.

Bulk tool dry runs validate syntax only, not remote existence/authorization. `rollbackOnError` compensates successful creates only; it does not undo edits to existing records, every audit event, or notifications. After timeout, cancellation, or partial failure, inspect per-item results and persisted state before choosing the next action.

For larger issue/project edits, use zero-based indices to build batches of 1–100 records. These examples are independent previews; set `--dry-run=false` only for the intended application:

| Command | Meaning |
| --- | --- |
| `nudge tool bulk_create_issues --issues-team-id 0=TEAM --issues-status-id 0=STATUS --issues-title '0=First task' --dry-run=true` | Create only; items must not contain `id`. |
| `nudge tool bulk_upsert_issues --issues-id 0=ISSUE --issues-priority 0=1 --issues-team-id 1=TEAM --issues-status-id 1=STATUS --issues-title 1=Follow-up --dry-run=true` | Patch an existing ID or create when ID is omitted; fields are directly on each item, not nested in `patch`. |
| `nudge tool bulk_upsert_projects --projects-id 0=PROJECT --unset-projects-target-date 0 --projects-key 1=NEXT --projects-name '1=Next release' --dry-run=true` | Apply mixed project patches/creates; supplied IDs must already exist. |

Inspect item `ok`, returned IDs, `rolledBack`, and `rollbackFailed`, not just aggregate `succeeded`. A successful create can subsequently have been deleted by compensation. Split larger inputs and keep ID mappings for follow-up parent links or recovery.
