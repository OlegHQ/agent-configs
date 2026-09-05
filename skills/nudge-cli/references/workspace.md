# Workspace setup, workflow configuration, views, and history

Use this reference for identity discovery, teams, issue statuses, labels, templates, saved boards/lists, and workspace audit questions. Replace uppercase IDs with discovered values and run the command for the requested operation. Repeated collection flags supply the complete replacement set; `--clear-FIELD` empties a collection and `--unset-FIELD` clears a nullable value. `--output json` controls results, not command input.

## Workspace access and people

```sh
nudge auth status
nudge tool get_access_context --output json
nudge tool list_actors --limit 50 --output json
nudge tool list_users --limit 50 --output json
nudge tool list_teams --limit 50 --output json
```

The access context identifies the credential's workspace, account, write permission, and optional project restriction. This is the active workspace; these tools do not switch workspaces. A project-scoped credential cannot list all workspace actors or read the workspace audit stream. Selecting a larger MCP toolset does not expand authorization.

Use `list_actors` for assignees, project leads/members, and actor filters: `kind` is `user` or `service_account`. `list_users` is specifically the human member directory. An agent's delegation target ID from `list_agents` differs from its `serviceAccountId`, which is the actor ID used for assignment. Follow each page's `nextCursor` with `--cursor` until absent; a first page is not the complete directory.

The generated tools expose member discovery, but not workspace creation/switching, invitations, role management, or API-token administration. For administration available through `nudge api` and features that require application settings, consult [administration](administration.md); do not invent generated commands or widen credential scope automatically.

## Search and overview

For “find everything about the rollout,” use the relevant resource searches and combine results with their resource kind and stable ID:

```sh
nudge tool list_issues --query rollout --limit 20 --output json
nudge tool list_projects --query rollout --limit 20 --output json
nudge tool search_documents --query rollout --limit 20 --output json
nudge tool get_user --id USER --output json
```

Issue search matches title/description, project search matches name/key, and document search matches title/markdown. Follow each result's continuation independently, then retrieve full records before editing or summarizing details absent from summaries. There is no single generated global-search command. `nudge overview` provides the legacy frontend overview JSON, not a complete export; use focused paginated reads for a complete inventory.

For command semantics, use `nudge tool search_documentation --query "saved views" --output json`. This searches the bundled usage guide, not workspace documents, and returns `structuredContent.results` instead of the ordinary API `structuredContent.data`. Command-specific `--help` remains the source for the installed executable's flags.

## Create a team and connect its project

For “set up a support team for this project,” first list teams and inspect the intended project to avoid creating a duplicate. Then adapt:

```sh
nudge tool create_team --key SUP --name Support --output json
nudge tool get_team --id TEAM --output json
nudge tool list_project_teams --project-id PROJECT --output json
nudge tool link_project_team --project-id PROJECT --team-id TEAM --output json
nudge tool list_issue_statuses --team-id TEAM --output json
```

Capture the created team's stable ID. A team key is not a team ID. `unlink_project_team` takes the same project/team fields and removes the association when that is the requested change; issues must be moved out of that team/project combination before unlinking. Read `get_team` before choosing issue estimates so values respect the team's enabled scale, extended range, and zero policy. Team creation is exposed; team rename/deletion are not exposed by the current generated CLI.

Estimate-policy configuration requires an unscoped workspace write credential:

```sh
nudge tool update_team_estimates --team-id TEAM --enabled --scale fibonacci --extended=false --allow-zero=false --unestimated-as-one=false --output json
```

This replaces the policy: supply every field and preserve current values outside the requested change. Valid scales are `exponential`, `fibonacci`, `linear`, and `t_shirt`. Re-read the team after updating.

## Configure team workflow statuses

For “add a review column between implementation and done,” list the team's statuses, reuse an existing matching status if appropriate, then create and position a new one:

```sh
nudge tool get_issue_status --team-id TEAM --id STATUS
nudge tool create_issue_status --team-id TEAM --status-name Review --status-category started --status-color '#336699'
nudge tool update_issue_status --team-id TEAM --id STATUS --position 2
nudge tool update_issue_status --team-id TEAM --id STATUS --name 'In review' --color '#663399'
```

Creation uses the default position; `--position 2` moves an existing status to zero-based position 2. To remove an unused status, run `nudge tool delete_issue_status --team-id TEAM --id STATUS`.

Categories are `backlog`, `unstarted`, `started`, `completed`, or `canceled`. A display name such as “Review” does not determine terminal behavior; the category does. The system-managed duplicate status is protected and is not an ordinary category to create or use for status moves. Configuration requires an unscoped workspace write credential. Before deleting a used status, inspect its issues and move only the intended work to another valid status; do not treat deletion as an implicit migration. Re-list statuses to verify ordering.

## Issue and project labels

```sh
nudge tool list_issue_labels --output json
nudge tool list_project_labels --output json
nudge tool create_issue_label --name Regression --color '#cc3344' --output json
nudge tool create_project_label --name Customer --color '#336699' --output json
```

The two label scopes are distinct. Read existing assignments before adding a label, then include every label to retain:

```sh
nudge tool update_issue --id ISSUE --labels EXISTING_LABEL --labels NEW_LABEL
nudge tool update_project --id PROJECT --label-ids EXISTING_PROJECT_LABEL --label-ids NEW_PROJECT_LABEL
nudge tool update_issue_template --id TEMPLATE --labels EXISTING_LABEL --labels NEW_LABEL
nudge tool update_label --id LABEL --color '#8844cc'
```

Repeated flags replace the complete assignment set. Use `--clear-labels` on issues/templates or `--clear-label-ids` on projects only when clearing all assignments is intended.

Both scopes use `nudge tool update_label --id LABEL --name 'New name'` to rename an unused label. Scope cannot be changed, and an assigned label cannot be renamed. `nudge tool delete_label --id LABEL` removes the label **and its workspace issue/template or project assignments**; use an unscoped workspace write credential and only delete when that wider cleanup is intended. Prefer stable IDs even though explicit issue/template writes accept workspace-local legacy names.

## Reusable issue templates

For “make a bug-report template,” list templates, resolve its team/status and any linked project/labels, then create defaults:

```sh
nudge tool list_issue_templates --output json
nudge tool get_issue_template --id TEMPLATE --output json
```

```sh
nudge tool create_issue_template --name 'Bug report' --team-id TEAM --title 'Bug: ' --description 'Describe reproduction, expected behavior, and actual behavior.' --priority 2 --status-id STATUS --labels LABEL
```

To detach optional planning defaults while preserving the wording:

```sh
nudge tool update_issue_template --id TEMPLATE --unset-project-id --unset-status-id --unset-estimate --clear-labels
```

`nudge tool delete_issue_template --id TEMPLATE` deletes the template. Templates require an unscoped workspace write credential to configure. Their status must belong to the chosen team, and their project must be linked to it. Priority is `0` none, `1` urgent, `2` high, `3` medium, `4` low; omitted create priority defaults to `2`.

To create an issue from a template in the current CLI, read the template and copy its applicable defaults into `create_issue` flags, replacing its title with the requested issue title:

```sh
nudge tool create_issue --team-id TEAM --status-id STATUS --title 'Bug: retries stop too early' --description 'Describe reproduction, expected behavior, and actual behavior.' --priority 2 --labels LABEL
```

Resolve a valid status if the template leaves one unset. There is no `apply_template` tool or `templateId` parameter on `create_issue`; creating a template alone does not create issues.

## Saved boards, lists, filters, and favorites

For “save an agent's urgent work as a board,” resolve the actor/team and inspect existing views:

```sh
nudge tool list_saved_views --output json
nudge tool get_saved_view --id VIEW --output json
```

```sh
nudge tool create_saved_view --name 'Urgent assigned work' --team-id TEAM --assignee-user-id ACTOR --priorities 1 --priorities 2 --layout board --scope active --grouping status --sort priority --show-sub-issues --favorite
```

Despite the flag's historical name, `--assignee-user-id` accepts both human and agent actor IDs. This stores presentation/filter settings; it does not assign or move issues. `--layout` is `board` or `list`; `--scope` is `active`, `backlog`, `all`, or `archived`; `--sort` is `position`, `priority`, `updated`, or `created`. `--grouping`/`--sub-grouping` can be `status`, `priority`, `assignee`, `project`, or `none`. Completion display uses `--completed-issues all` or `--completed-issues hide`.

Use `update_saved_view` for a precise change:

```sh
nudge tool update_saved_view --id VIEW --layout list --unset-assignee-user-id --clear-priorities --favorite=false
```

Omitted settings remain unchanged. Use `--unset-project-id`, `--unset-team-id`, or `--unset-assignee-user-id` to remove a single filter, and `--clear-priorities`, `--clear-status-ids`, or `--clear-labels` to clear a collection. Explicit false disables boolean settings. Read `get_saved_view` before editing advanced filters or display properties, and inspect current help rather than assuming arbitrary filter fields/operators are supported. A saved view is separate from a document database view; database views live in the document's schema.

```sh
nudge tool duplicate_saved_view --id VIEW --name 'Urgent work copy'
nudge tool delete_saved_view --id VIEW
```

These are independent operations. Duplication copies settings with a new ID and favorite reset to false; deletion removes the saved configuration. Re-read the resulting view after a write; duplicating a view does not duplicate its issues.

## Audit and activity investigations

For “what changed in this workspace,” use the audit stream:

```sh
nudge tool list_audit_events --limit 50 --output json
nudge tool list_audit_events --limit 50 --cursor CURSOR --output json
```

It requires an unscoped workspace credential and returns newest-first events, with stable ID ordering for timestamp ties. Continue until the requested period or absent cursor; there is no tool-level date, actor, or entity filter, so filter returned events locally as needed. A removed cursor event requires restarting at page one. New events during paging mean this is not a frozen snapshot; deduplicate by event ID when combining repeated passes.

For a narrower history, choose the relevant supported source: `list_comments --issue-id ISSUE` for discussion, `list_project_updates --project-id PROJECT` for progress reports, `list_document_revisions --id DOCUMENT` for document snapshots, or `get_agent_session --session-id SESSION` for delegation activity. Agent history uses `nextAfterSequence`/`--after-sequence` until `activityPageComplete`, rather than an ordinary cursor. These sources answer different questions; comments alone do not prove the complete edit history.

The current CLI has no general activity-feed, notification-inbox, or mark-notification-read tool. Use the audit and entity history that actually exists and state any missing coverage when answering a historical question. Reads do not repair pending comment audit receipts or agent history; that requires the exact documented mutation retry, with the original key and payload, when recovery is part of the authorized task.
