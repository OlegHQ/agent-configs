# Same-origin frontend integration

Use this reference to connect the static Next.js frontend to the Go API without frontend deployment configuration.

## Contents

- shadcn/ui and React Query
- URL-owned workspace context
- Relative browser requests and development proxying
- Public runtime configuration
- Static export embedding
- Browser identity

## Build UI with shadcn/ui

Product UI in `ui/` uses shadcn/ui components copied into `components/ui/`. Follow [shadcn-ui.md](shadcn-ui.md) and the project **shadcn** skill for CLI usage, composition, and styling rules. Add global providers once in `app/layout.tsx`.

## Keep browser requests relative

Use relative URLs everywhere:

```ts
fetch("/api/posts")
fetch("/api/config")
```

Do not create an API base URL abstraction, read frontend environment variables, inspect `window.location` to reconstruct the origin, or embed deployment hostnames. Relative URLs naturally target the page's origin and preserve same-origin cookies and browser security behavior.

## Manage remote state with React Query

Install and use `@tanstack/react-query` for data owned by the Go API. Mount one `QueryClientProvider` in the application provider tree and keep the `QueryClient` stable for the browser session. Centralize stable, hierarchical query keys by component so mutations can update or invalidate the exact affected cache entries.

Use `useQuery` for reads and `useMutation` for writes. On successful writes, update the cache directly when the response is authoritative and small; otherwise invalidate the affected keys. Keep transient form input in local React state, but do not mirror query results, loading flags, request errors, or mutation status in `useState`/`useEffect`. Configure retry behavior deliberately, especially for authorization, validation, and not-found responses.

Keep the typed same-origin API client as the query function boundary. React Query owns request lifecycle and server-state caching; it does not replace API response validation or backend authorization.

Primary sources: [TanStack Query installation](https://tanstack.com/query/latest/docs/framework/react/installation), [QueryClientProvider](https://tanstack.com/query/latest/docs/framework/react/reference/QueryClientProvider), [useQuery](https://tanstack.com/query/latest/docs/framework/react/reference/useQuery), and [useMutation](https://tanstack.com/query/latest/docs/framework/react/reference/useMutation).

In production, the Go binary serves both `/api/*` and embedded frontend assets. Register `/api/config` and all API routes before static middleware or the SPA fallback. Do not enable broad CORS for this topology.

## Make the URL workspace authoritative

Represent the active workspace on every workspace-owned page with a `workspace=<slug>` query parameter. Keep navigable temporary state such as selected entities, tabs, saved views, filters, layout modes, and archive state in query parameters beside it. Reserve component state for ephemeral interaction details that should not survive reloads or produce shareable URLs.

Resolve the slug against the authenticated user's accessible workspaces before mounting the application shell or any workspace-owned query. Send that slug with every browser API request and authorize it against the user's active membership at the backend boundary; never rely exclusively on a mutable current-workspace cookie, because another browser tab can change it. When the URL differs from the session workspace, show a neutral transition state, select the requested workspace through the backend, clear the previous workspace's cached server data, refresh the authenticated principal, and only then mount the page. Reject inaccessible or unknown workspace slugs without rendering data from the session's previous workspace.

Canonicalize legacy workspace pages that omit the parameter by adding the session's current accessible workspace slug while preserving all other query parameters. Preserve the workspace parameter in page navigation. When explicitly switching workspaces, change the URL first and clear entity identifiers that belong to the previous workspace; let the same reconciliation boundary update the session. This keeps reloads, copied URLs, browser history, the visible workspace name, and fetched records in agreement.

## Proxy only during Next development

The Next development server and Go server use different local ports. Keep browser calls relative and proxy the API path from Next to one fixed, documented local Go origin, for example `http://127.0.0.1:8080`.

Use a phase-aware `next.config`:

```js
const { PHASE_DEVELOPMENT_SERVER } = require("next/constants")

module.exports = (phase) => {
  if (phase === PHASE_DEVELOPMENT_SERVER) {
    return {
      async rewrites() {
        return [{
          source: "/api/:path*",
          destination: "http://127.0.0.1:8080/api/:path*",
        }]
      },
    }
  }

  return { output: "export" }
}
```

Adapt syntax to the repository's Next.js version and module format. Keep the destination fixed in central development tooling rather than introducing frontend application configuration. Proxy the same `/api` path without renaming it so development and production behavior match.

Next.js documents rewrites as URL proxies, but rewrites and Proxy require a Next server and are unsupported in static exports. Use them only in `PHASE_DEVELOPMENT_SERVER`.

Primary sources: [Next.js rewrites](https://nextjs.org/docs/app/api-reference/config/next-config-js/rewrites), [configuration phases](https://nextjs.org/docs/app/api-reference/config/next-config-js), and [static export limitations](https://nextjs.org/docs/app/guides/static-exports).

## Serve public runtime configuration from Go

Expose one endpoint:

```text
GET /api/config
```

Return a dedicated response type, not the backend `Config` struct. Allowlist only values required by browser behavior, for example:

```json
{
  "schema_version": 1,
  "features": {
    "new_editor": true
  },
  "limits": {
    "upload_bytes": 10485760
  }
}
```

Treat the entire response as public. Never expose secrets, Vault metadata, internal hostnames, database details, signing settings, or authorization policy internals. Keep user-specific configuration in authenticated user endpoints rather than this application bootstrap document.

Derive the response from the backend's final typed and validated configuration. Construct it explicitly at the transport boundary. Version the schema when compatibility matters and generate shared transport types from the API schema when the project already uses generation.

Choose cache semantics deliberately. Default to `Cache-Control: no-store` for a small startup document so a newly deployed backend cannot be paired with stale configuration. Use ETag revalidation only when request volume demonstrates a need and test deployment behavior.

## Load configuration once

Create one typed frontend loader and one application-level provider/store. Fetch `/api/config` during client bootstrap before features that require it are rendered.

- Validate the response shape at the network boundary.
- Represent loading and failure states explicitly.
- Retry only with bounded backoff when appropriate.
- Do not silently substitute build-time defaults after a fetch failure.
- Do not let individual components fetch configuration independently.
- Keep API clients relative even after configuration loads; the response must not contain an API origin.

Test the backend allowlist and secret exclusion, response schema, cache headers, route precedence, frontend parsing, bootstrap failure UI, and development rewrite. Include one browser or integration smoke test proving that `/api/config` and another `/api/*` request work through the Next development origin and through the built Go binary.

## Serve the static export from Go

After `output: "export"`, the Go binary serves the copied `ui/out` tree from an embedded `fs.FS`.

Requirements:

1. Stage the full export with `cp -a ui/out/. backend/internal/infrastructure/webui/dist/` (or the repository's equivalent path).
2. Embed with `//go:embed all:dist` so nested `_next/static/media` font files are included.
3. Register API and health routes before static middleware.
4. Use SPA fallback only for HTML navigation routes. Return `404` for missing `/_next/*`, `.css`, `.js`, `.woff2`, images, and other static assets. If a missing stylesheet returns `index.html`, the browser ignores it and the page renders with default serif fonts and no Tailwind layout.
5. Run `scripts/verify-ui-export.sh ui/out` after staging to confirm:
   - `index.html` stylesheet links resolve to files on disk
   - `_next/static/media/*.woff2` exists
   - `@font-face` URLs referenced from exported CSS resolve under `_next/static/media/`

Smoke-test at least one CSS chunk, one font file, one missing static asset (`404`, not HTML), and one SPA route through the Go binary before release.

## Ship a complete browser identity

Treat favicon and install metadata as production assets, not starter-project decoration. Generate the asset family from one canonical brand source with a repository script rather than hand-editing binary files. Include an SVG icon, `favicon.ico`, 16×16 and 32×32 PNG favicons, a 180×180 Apple touch icon, 192×192 and 512×512 PWA icons, a 512×512 maskable icon, a Safari pinned-tab SVG, and `site.webmanifest`.

Declare the manifest, standard icons, shortcut icon, Apple icon, Safari mask color, and platform tile color through Next.js metadata. Remove framework starter marks such as `next.svg` and `vercel.svg`. Make the export verifier fail when any required asset or metadata link is missing.

After embedding, probe the public manifest and representative icon variants. Require successful responses with image/manifest content types and reject HTML fallback bodies for asset paths.
