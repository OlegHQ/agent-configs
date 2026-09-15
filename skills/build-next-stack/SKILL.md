---
name: build-next-stack
description: Structure, implement, refactor, or review a Next.js, React, Tailwind, and shadcn/ui frontend in ui/ that depends on the build-go-stack backend foundation. Use for Next.js pages and components, Bun tooling, shadcn composition, React Query server state, same-origin API clients, backend-served public configuration, development proxying, static export, browser identity, frontend staging, Go embedding integration, frontend checks, or the complete single-binary application artifact. Always apply build-go-stack first; this skill extends it and never replaces or duplicates its Go architecture guidance.
---

# Build Next Stack

Build the static Next.js frontend as a dependent layer on the Go application foundation.

## Apply the required Go foundation

1. Read and apply the draft [`build-go-stack`](../../drafts/build-go-stack/SKILL.md) skill in full before planning or changing a Next.js application. This dependency is mandatory even for frontend-only tasks because API contracts, actors, configuration, verification order, and artifact ownership cross the boundary.
2. Keep `build-go-stack` authoritative for Go packages, backend APIs, configuration, logging, errors, MCP, and backend tests.
3. Keep this skill authoritative for `ui/`, browser behavior, Next.js static export, and the frontend-to-Go integration seam.
4. Resolve overlap by dependency direction: this skill may constrain how the frontend consumes or is staged into Go; it must not redefine backend architecture.

## Start with frontend discovery

1. Inspect repository instructions, `ui/package.json`, `bun.lock`, `next.config.*`, `components.json`, frontend build targets, API contracts, and existing conventions.
2. Preserve compatible choices and avoid unrelated visual or package churn.
3. Read [references/frontend-integration.md](references/frontend-integration.md) before changing API clients, React Query, frontend configuration, development rewrites, CORS, static export, Go embedding, or browser identity.
4. Read [references/shadcn-ui.md](references/shadcn-ui.md) and apply the **shadcn** skill before changing components, forms, dialogs, navigation, or shared visual patterns.
5. Read [references/performance.md](references/performance.md) before changing Bun/Next scripts, UI checks, staging, fingerprints, caches, or the combined artifact pipeline.
6. Apply **vercel-react-best-practices** for React/Next implementation or performance work and **vercel-composition-patterns** for reusable component APIs, provider design, compound components, or boolean-prop proliferation.
7. Apply **frontend-design** for a new or substantially reshaped visual direction, plus the product-specific UI skill when one exists.

## Use the frontend layout

```text
ui/
  app/ or pages/
  components/
    ui/                 # shadcn CLI-managed primitives
    <feature>/          # feature composition
  components.json
  lib/
  public/
  next.config.*
  package.json
  bun.lock
```

- Manage the frontend with Bun: `bun install`, `bun run`, and `bunx --bun`. Commit `bun.lock`; do not introduce npm, pnpm, or yarn lockfiles.
- Keep global providers in one client wrapper imported by `app/layout.tsx`.
- Add `"use client"` only where browser APIs, hooks, or event handlers require it.
- Use `@tanstack/react-query` for server state with one stable application-level `QueryClient`, hierarchical query keys, and targeted updates or invalidation.
- Keep transient form state local; do not mirror query results, loading flags, request errors, or mutation state through effects.
- Put navigable page state in URL query parameters, including the workspace slug, selected entity, tab, view, filters, and archive mode. Treat the URL workspace as authoritative: do not mount workspace-owned queries until the authenticated session has been reconciled to it.
- Start independent requests together, avoid sequential client waterfalls, import modules directly instead of through broad barrels, and dynamically load genuinely heavy optional UI.
- Derive render state during render rather than synchronizing it with effects. Move interaction logic to event handlers and memoize only measured expensive work.

## Build UI with shadcn/ui

- Use the repository's existing shadcn preset, or `base-nova` when initializing a new project.
- Use the shadcn CLI to inspect and add components. Compose feature UI from primitives before creating custom controls.
- Inspect `components.json` and run the project's package-runner form of `shadcn info` before assuming the base, aliases, icon library, or installed components. Read current component docs before implementing an API from memory.
- Keep CLI-managed primitives in `components/ui/` and application composition in feature directories.
- Use semantic theme tokens and map Tailwind font tokens to `next/font` variables without circular CSS variable references.
- Prefer built-in variants, preserve required group wrappers, give every overlay an accessible title, include avatar fallbacks, and follow the configured icon library and base-specific trigger API.
- Scope global shortcuts away from form controls and descendants of editable content.
- Give primary entities dedicated shareable routes. Reserve dialogs, sheets, and drawers for bounded transient work.
- Replace proliferating boolean mode props with explicit variants or composed subcomponents. Let providers expose state, actions, and metadata without leaking their storage mechanism.
- For visual redesigns, ground the direction in the product's subject and audience, choose a coherent type/color/layout system, make one justified signature move, and validate keyboard focus, reduced motion, and mobile behavior.

## Keep the browser same-origin

- Call relative routes such as `/api/issues` and `/api/config`; never construct or configure a frontend API origin.
- Use a development-only Next rewrite to the fixed local Go server while preserving `/api/*` in the browser.
- Do not use frontend environment variables, `NEXT_PUBLIC_*`, runtime config files, injected globals, or build-time substitutions for application configuration.
- Load one typed, allowlisted public document from the Go `GET /api/config` endpoint. Treat every field as public and handle bootstrap failure explicitly.
- Do not add production CORS for the same-origin deployment.
- For this static export, scope workspace pages with a `workspace=<slug>` query parameter instead of runtime-only dynamic routes. Preserve that parameter in navigation and keep other temporary navigation state in the query string.

## Export and integrate one artifact

1. Configure `output: "export"` and a stable output directory such as `ui/out`.
2. Avoid runtime-only Next.js features: Server Actions, request-time route handlers, redirects, headers, ISR, Proxy, and the default image optimizer.
3. Copy the complete export beneath the Go module into a generated staging directory suitable for `go:embed`; never embed across module boundaries.
4. Embed with `//go:embed all:dist` and serve through Fiber's maintained static middleware.
5. Register API, health, metrics, and MCP routes before frontend fallback.
6. Restrict SPA fallback to HTML navigation. Missing scripts, styles, fonts, images, and `/_next/*` assets must return `404`, never `index.html`.
7. Verify exported CSS, referenced fonts, manifest, icons, and representative content types before compiling the final Go binary.
8. Fail the complete artifact build when the export is missing or stale. Never silently ship an empty or outdated UI.

Centralize UI build, verification, staging, and final artifact commands in the repository's existing build tool. Keep backend-only, UI-only, fast-check, and complete-artifact targets distinct. The normal Go edit loop must not rebuild Next.js.

## Verify proportionally

- Follow repository validation order: Oxlint first, TypeScript when contracts/types changed, rare focused RTL only for high-risk interactions, then production export/staging when frontend inputs changed.
- Use targeted browser inspection only when static checks cannot falsify behavior. Keep the scripted Playwright suite for the final regression/release gate or changes to its coverage.
- Preserve Bun and `.next/cache`; install worktree-local dependencies and never symlink `node_modules` outside the worktree.
- Run the complete single-binary artifact gate when integration/staging inputs changed or before release handoff.
- Smoke-test API precedence, one CSS chunk, one font, a frontend route, a missing static asset returning `404`, and public configuration through the built Go binary when the integration seam changes.
- Report commands, timing, failures, assumptions, and skipped checks.

## Reject recurring failure modes

- Do not use a Next.js production server when the deployment artifact is the Go binary.
- Do not use server-only Next.js features in a static-export application.
- Do not add an API base URL, production CORS, or frontend application configuration.
- Do not hand-manage server state or duplicate React Query lifecycle state.
- Do not hand-roll common UI controls when an installed shadcn primitive or composition fits.
- Do not make overlays the canonical detail experience for primary entities.
- Do not omit CSS, fonts, icons, or other nested export assets from staging.
- Do not allow static fallback to swallow API routes or missing assets.
- Do not make backend-only checks rebuild an unchanged frontend or replace staged files whose contents are unchanged.
- Do not alter Go architecture here; apply `build-go-stack` for backend changes.
