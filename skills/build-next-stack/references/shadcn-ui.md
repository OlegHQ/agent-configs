# shadcn/ui in the Next stack

Use **shadcn/ui** as the default component system for `ui/`. Components are copied into the repository as source under `components/ui/` and composed into feature pages and layouts.

## When to read this

Before adding UI surfaces, forms, dialogs, navigation shells, command palettes, tables, or shared visual patterns in `ui/`.

Also follow the **shadcn** skill for CLI commands, composition rules, styling constraints, and component selection.

## Project conventions

- Initialize with `bunx --bun shadcn@latest init` inside `ui/`. Commit `ui/components.json`.
- Run `bunx --bun shadcn@latest info` before assuming aliases, component base, icon library, Tailwind version, or installed components. Run `docs <component>` before implementing against a component API from memory.
- Add components with `bunx --bun shadcn@latest add <name>`. Prefer composing existing components before writing custom markup.
- Preview updates to installed components with `--dry-run` and `--diff`; never overwrite local component changes without explicit approval.
- Keep aliases from `components.json` (`@/components/ui`, `@/lib/utils`, etc.). Do not hardcode alternate import paths.
- Theme tokens live in `app/globals.css`. Edit that file for CSS variables; do not create parallel theme files.
- When using `next/font`, map Tailwind font tokens to the font variable classes applied on `<html>`:

```css
@theme inline {
  --font-sans: var(--font-geist-sans);
  --font-mono: var(--font-geist-mono);
  --font-heading: var(--font-geist-sans);
}
```

```tsx
const geistSans = Geist({ variable: "--font-geist-sans", subsets: ["latin"] })

<html className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}>
```

Never write `--font-sans: var(--font-sans)`; the circular reference breaks `font-sans` utilities and falls back to browser serif defaults.
- Use semantic Tailwind tokens (`bg-background`, `text-muted-foreground`, `border-border`). Do not override component colors with raw palette classes.
- Use `FieldGroup` + `Field` for forms, `Sidebar` for app navigation, `Command` inside `Dialog` for command palettes, `Table` + `Badge` + `Avatar` for issue lists, and `Dialog`/`Sheet` for modals.
- Keep items inside required groups, give every dialog/sheet/drawer a title, include `AvatarFallback`, and use the base-specific `asChild` or `render` trigger API reported by `info`.
- Treat routing as part of the product UI. Primary entity details belong on full pages with stable, shareable URLs and browser navigation. Use a wide content column plus a stable metadata/action rail when the record combines narrative content, activity, and properties. Keep dialogs and sheets for bounded transient work rather than canonical issue, project, document, or initiative details.
- Wrap the app with required providers once in `app/layout.tsx` (for example `TooltipProvider`, `Toaster`).
- Add `"use client"` only where browser APIs, hooks, or event handlers require it. Respect `components.json` `rsc` settings.

## Layout shape

```text
ui/
  app/
    layout.tsx          # global providers
    globals.css         # shadcn theme tokens
  components/
    ui/                 # shadcn primitives (CLI-managed)
    ...                 # feature-specific composition (hand-written)
  components.json
  lib/utils.ts          # cn()
```

Feature components belong beside their component (`components/issues/`, `components/shell/`, etc.). Keep shadcn primitives in `components/ui/` and compose upward.

## Static export constraints

shadcn components work with Next static export when client boundaries are explicit. Do not rely on Next server-only features for UI behavior. Fetch data from relative `/api/*` routes in client components or during client bootstrap.

Static export must include fonts and stylesheets:

- Next emits CSS under `_next/static/chunks/*.css` and fonts under `_next/static/media/*.woff2`.
- The Go embed step must copy the entire `ui/out/` tree; do not stage only HTML/JS and omit `media/`.
- Verify the export with `scripts/verify-ui-export.sh ui/out` before embedding.

## CLI quick reference

```bash
cd ui
bunx --bun shadcn@latest info
bunx --bun shadcn@latest search @shadcn -q "sidebar"
bunx --bun shadcn@latest add button dialog sidebar command
bunx --bun shadcn@latest docs button dialog
```

Use `bunx --bun shadcn@latest docs <component>` before implementing or fixing a component API. Run `info` when aliases, base library, or icon library may have changed.

## Do not

- Hand-roll styled divs when an installed shadcn component fits.
- Use `space-x-*` / `space-y-*` for layout; use `flex` + `gap-*`.
- Add npm/pnpm/yarn shadcn invocations in docs or scripts for this stack; use Bun.
- Edit generated shadcn component files for one-off styling when a variant or composition change is enough.
