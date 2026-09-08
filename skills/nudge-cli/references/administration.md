# API-only operations and browser handoffs

Use this reference for what the CLI cannot do, the raw `nudge api` escape hatch, and credential hygiene for automation.

## Raw API escape hatch

```sh
nudge api GET /issues/NUD-7
nudge api POST /issues --data '{"teamId":"team_...","title":"..."}'
nudge api GET /audit-events/page --query limit=10
```

`nudge api METHOD PATH [--data JSON|--input FILE] [--query k=v]` calls any route on the configured origin for cases with no dedicated command. PATH may be relative (`/issues`) or absolute (`/api/v1/issues`); it always stays under `/api/v1` on the configured origin. The body comes from `--data` (inline JSON) or `--input` (a file, or `-` for stdin); GET/DELETE usually need neither. It returns raw API JSON, not a `Next steps`-carrying envelope. Prefer a dedicated command whenever one exists — this is the fallback, not the default interface.

## Authentication boundary

Nudge CLI authentication uses service-account bearer tokens. A token does not turn the caller into the human who created it, even an administrator — product operations that require a human session need the signed-in web app. Do not suggest repeatedly logging into the CLI, an invented admin scope, or exporting browser cookies to bypass this. Prefer a dedicated service-account token (`nudge access rotate`) over a personal/human token for any automation or agent credential, and never echo a token in chat, logs, or command output beyond the one-time `nudge access rotate` result (see [workspace workflows](workspace.md#token-lifecycle) for the safe pipe pattern).

## What requires the web app

| Task | Why the CLI can't do it |
| --- | --- |
| Create/switch/delete a workspace | No workspace-switch command; a token is bound to its account's workspace. Obtain a token for the target workspace instead. |
| Invite members, change roles, activate/deactivate membership, manage invitations | Requires an unscoped human administrator session; `nudge actor list` is discovery only, not member administration. |
| Change workspace appearance (theme/accent/surface/background) | Requires a human administrator in the web app; reading it is API-accessible (below). |
| Upload or change a profile picture, upload media through a UI | Media upload is multipart; `nudge api --data` only sends JSON. Profile edits also require a human principal. |
| Read or mark notifications read | The notification routes require a human principal; service-account tokens cannot use them. |
| Create/edit an agent's identity (name, avatar, description) or enable delegation | Requires an unscoped human administrator session in the web app; running an already-eligible agent session is the part the CLI does (`nudge agent ...`). |

Team creation, workflow statuses, project-team links, and estimate-policy changes ARE available from the CLI — see [workspace workflows](workspace.md).

## Read-only admin routes via `nudge api`

```sh
nudge api GET /workspace/appearance
```

Returns `mode`, `accent`, `surface`, and `background`. Changing it still requires the web app (above).

```sh
nudge media delete MEDIA
```

Media has a dedicated command, not just `nudge api`: `nudge media delete MEDIA` permanently deletes an uploaded media item owned by the current service account. Resolve the actual media ID from wherever it was referenced — an issue attachment ID is a different resource.

## Document recovery

Covered in full under [documents](workflows.md#documents) with the rest of the document surface: `nudge document archive`/`restore` for the whole tree, and `nudge document revision list`/`get`/`restore` for revision-level recovery. All of these are ordinary CLI commands now, not API-only operations.

## A failed call does not expand authority

If a requested operation is unavailable to the current credential (exit code 6, forbidden) or unavailable in the CLI at all, say so plainly and name the actual supported path (a different route, a web-app step, or a wider-scoped credential to request) rather than working around it.
