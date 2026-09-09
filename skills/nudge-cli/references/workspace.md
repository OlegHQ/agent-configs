# Workspace setup, workflow configuration, views, and history

Repeated collection flags supply the complete replacement set; resource-specific `--clear-*` flags empty a collection, and `none` clears a single nullable value. `-o json` controls the output format, not command input. Replace uppercase placeholders with discovered IDs/keys.

## Identity and access

```sh
nudge whoami
nudge auth status
nudge actor list
nudge actor list --kind service_account
nudge actor get ACTOR
```

`whoami` shows the current credential's workspace, account, and write access — this is the active workspace; there is no CLI workspace-switch command. `actor list --kind` filters to `user` or `service_account`; an agent's actor ID (used for issue assignment) differs from its delegation ID used by `nudge agent delegate --agent`.

## Teams and estimate policy

```sh
nudge team list
nudge team get NUD
nudge team create --key NUD --name Nudge
nudge team estimates NUD
nudge team estimates NUD --scale fibonacci
nudge team estimates NUD --enabled=false
```

`team estimates` without flags prints the current policy; with any flag, the policy is replaced entirely — `--enabled`, `--scale` (`exponential`, `fibonacci`, `linear`, `t_shirt`), `--extended`, `--allow-zero`, `--unestimated-as-one` — unset flags keep the team's current values. There is no team rename/delete command in the CLI.

## Workflow statuses

```sh
nudge status list NUD
nudge status get NUD STATUS
nudge status create NUD --name Review --category started --color "#336699"
nudge status update NUD STATUS --position 2
nudge status update NUD STATUS --name "In review" --color "#663399"
nudge status delete NUD STATUS
```

`--category` is `backlog`, `unstarted`, `started`, `completed`, or `canceled` — the display name does not determine terminal behavior, the category does. `status update --position` moves an existing status to that zero-based position within the team's workflow. Before deleting a used status, move its issues to another valid status first; deletion does not migrate them.

## Labels

```sh
nudge label list
nudge label list --scope project
nudge label list --scope all
nudge label create --name bug --color "#ef4444"
nudge label create --name "Q4 Roadmap" --scope project
nudge label update LABEL --name "New name"
nudge label update LABEL --color "#8844cc"
nudge label delete LABEL
```

Labels belong to either the `issue` or `project` scope (default `issue`); `--scope all` on `label list` merges both scopes and ignores `--limit`/`--cursor`/`--all`. A name that exists in both scopes needs `--scope` to disambiguate on `label update`/`label delete`. Deleting a label also removes it from every issue/template or project it is assigned to.

## Issue templates

```sh
nudge template list
nudge template get TEMPLATE
nudge template create --team NUD --name Bug --title "Bug: " --priority high
nudge template create --team NUD --name Chore --title Chore --status Todo --label chore
nudge template update TEMPLATE --title "Fixed: " --priority urgent
nudge template update TEMPLATE --clear-labels --status none
nudge template delete TEMPLATE
```

A template's `--status` must belong to its team, and its `--project` must be linked to that team. There is no `apply-template`/`create-from-template` command: to create an issue from a template, read the template with `nudge template get` and copy its defaults into `nudge issue create` flags, replacing the title with the requested one.

## Saved views

```sh
nudge view list
nudge view get VIEW
nudge view create --name "My work" --assignee me --layout board
nudge view create --name Backlog --team NUD --status-id Backlog --input filters.json
nudge view update VIEW --layout list --clear-priorities --favorite=false
nudge view duplicate VIEW --name "My work (copy)"
nudge view delete VIEW
```

`--layout` is `list` or `board`; `--priority` (repeatable) and `--status-id` (repeatable) each replace the view's full filter list, cleared with `--clear-priorities`/`--clear-status-ids`; `--clear-labels` clears the label filter. `--assignee`, `--project`, and `--team` accept `none` to remove that single filter. `--input FILE` sets filters/display properties directly from a JSON object (see [argument handling](arguments.md#saved-view-filter-shape) for the filter shape); flags override the matching keys. `view duplicate` copies settings under a new name with favorite reset to false — it does not duplicate the underlying issues.

## Audit

```sh
nudge audit list
nudge audit list --limit 50 --cursor CURSOR
```

Returns newest-first events; there is no date/actor/entity filter, so filter returned events locally. For narrower history use the relevant resource instead: `nudge issue comment list ISSUE`, `nudge project status-update list PROJECT`, `nudge document revision list DOCUMENT`, or `nudge agent session get SESSION`.

## Token lifecycle

```sh
nudge access rotate
nudge access revoke
```

`access rotate` invalidates the current token and returns the new secret once, in the `token` field of its output — never print or paste this into chat or logs. Capture it in a private temporary file before login so a failed save does not lose the only copy:

```sh
umask 077
rotation_dir=$(mktemp -d)
nudge access rotate -o json > "$rotation_dir/rotation.json"
# Continue only if rotation succeeded.
jq -er .data.token "$rotation_dir/rotation.json" | nudge auth login --with-token
# Keep the private response until the replacement credential is verified.
```

Use the same `--url` for both commands on a non-default origin. If `NUDGE_API_TOKEN` supplies the old credential, update its authorized secret source too — `auth login` only updates the saved credential. Verify `nudge whoami` with the replacement credential, then remove the temporary response. Do not blindly repeat a timed-out rotation: it may have committed. Recover a lost replacement through an authorized human administrator in the web app. `access revoke` immediately ends the current credential's access and cannot be undone from the CLI; on a terminal it asks you to type "revoke", in scripts pass `--yes`. Prefer a dedicated service-account token (rotated with `access rotate`) for automation over a personal/human session token.
