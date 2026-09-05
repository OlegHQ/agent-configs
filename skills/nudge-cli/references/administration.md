# API-only operations and browser handoffs

Use this reference for document recovery, credential lifecycle, workspace administration, inbox, appearance, profile, and media. Use generated flags for document recovery and team settings. The remaining raw API operations below have no request body.

## Raw API contract and authentication boundary

`nudge api METHOD /api/v1/PATH` is a legacy escape hatch for known routes, not the normal command interface. Its optional `--data` accepts raw JSON only; it does not offer typed body flags, `--input`, multipart upload, custom-header, or cookie-session support. It returns raw API JSON, without the generated commands' MCP envelope. Paths must stay under `/api/v1/` on the configured origin. It cannot call `/api/auth/login`, `/healthz`, or a remote attachment URL. Prefer the generated command whenever available.

Nudge CLI authentication uses service-account bearer tokens. It does not turn the caller into the human who created the token, even when that human is an administrator. Product operations requiring a human session therefore need the signed-in web app. Do not suggest repeatedly logging into the CLI, an invented admin scope, or exporting browser cookies to bypass this distinction.

## Document recovery

Read the document's current `updatedAtMs` and the actual revision ID before recovery:

```sh
nudge tool search_documents --query Runbook --include-archived --output json
nudge tool get_document --id DOCUMENT --output json
nudge tool list_document_revisions --id DOCUMENT --limit 5 --output json
nudge tool restore_document --id DOCUMENT --expected-updated-at-ms 1700000000000
nudge tool restore_document_revision --id DOCUMENT --revision-id REVISION --expected-updated-at-ms 1700000000000
```

These are separate operations, not a prescribed sequence. Replace the sample timestamp with the latest value for the chosen operation. Unarchiving restores the authorized descendant tree and detaches the root if its parent is missing or archived. Tree restoration is nontransactional: inspect state after partial failure. Revision restoration requires the version field and appends a new revision; it preserves historical revisions rather than erasing them. Read the document again between successive mutations to obtain its new version.

## Service-account token lifecycle

`get_access_context` identifies the current account, scope, identity, and write access. The raw endpoints `POST /api/v1/access/rotate` and `POST /api/v1/access/revoke` operate on that same service account. Write permission is required; a service account cannot rotate or revoke a different account.

Rotation invalidates the old token and returns the new secret in `token`; `nudge auth logout` only deletes the local credential and does not revoke it. When rotation is requested, capture the response into a restricted local file rather than displaying it in tool output or chat. For example, on a POSIX shell, after creating a private temporary directory:

```sh
umask 077
token_dir=$(mktemp -d)
nudge api POST /api/v1/access/rotate --output json > "$token_dir/rotation.json"
jq -er '.token' "$token_dir/rotation.json" | nudge auth login --with-token
```

Use the same `--url` for rotation and login on a nondefault instance. If `NUDGE_API_TOKEN` supplies the old credential, update its authorized secret source too: saving a credential does not replace that environment variable. Verify access using the new credential before removing the protected response file. Do not replay a timed-out rotation blindly: it may already have invalidated the old credential. If the replacement was lost, recover through an authorized human administrator in the web app. Revocation likewise makes subsequent requests with that token fail.

Creating accounts, listing the account administration catalog, editing an agent's identity, and enabling delegation require an unscoped human administrator session. In the web app, configure a service account with `read` or `write` access and `workspace` or `project` resource scope; project scope needs the intended project. Agent identity configuration uses name, avatar, and description, then delegation can be enabled. These setup steps are distinct from running an already eligible agent session through the CLI.

## Workspace, membership, and invitations

Use the web app for workspace creation, switching, and deletion; the CLI has no workspace-switch command. A token remains bound to its account's workspace. To work in another workspace from the CLI, obtain an authorized token for that workspace and use the normal credential workflow.

Member administration and invitations require an unscoped human administrator. The product supports listing members, changing `admin`/`member` roles, activating/deactivating membership, listing invitations, inviting by email and role, resending, and revoking invitations. Membership mutations use the membership ID, which is distinct from the user/actor ID used in assignments. Invitation creation/resend returns a secret invitation token; handle it through the authorized invitation workflow without publishing it in logs or chat. The CLI's `list_users` and `list_actors` are discovery surfaces, not substitutes for member administration.

## Inbox, appearance, profile, and media

| User task | Supported path |
| --- | --- |
| Read notifications or mark one/all read | Human inbox in the web app. The API's notification routes require a human principal; service-account tokens cannot use them. |
| Subscribe/unsubscribe to issue activity | CLI `subscribe_to_issue`, `unsubscribe_from_issue`, and `get_issue_subscription` support both humans and service accounts at the product boundary; with CLI authentication they act on the current service account. Subscription does not give it a human inbox. |
| Read workspace appearance | `nudge api GET /api/v1/workspace/appearance` returns `mode`, `accent`, `surface`, and `background`. |
| Change workspace appearance | Human administrator in the web app. This updates workspace settings, not merely terminal output styling. |
| Upload an image or choose a profile picture | Use the web app. Media upload is multipart and unsupported by `nudge api --data`; profile edits require a human principal and a profile image uploaded by that user. |
| Add a remote resource to an issue | Use `create_attachment` with its HTTPS resource URL; this creates a link and does not upload the file. |
| Delete previously uploaded media | `nudge api DELETE /api/v1/media/MEDIA` can delete media owned by the current service account, subject to API authorization. Resolve the actual media ID from the upload result; an issue attachment ID is a different resource. |

Team creation, workflow statuses, project-team membership, and typed estimate-policy commands are covered in [workspace configuration](workspace.md). A failed API call does not expand the token's authority; report the specific supported web-app step when the requested feature requires a human session.
