# Fast Next.js checks, export, and staging

Use this reference to keep frontend iteration independent from Go while preserving a correct combined artifact.

## Keep targets separate

```text
ui-lint       -> UI source and lint configuration
ui-typecheck  -> UI source and TypeScript configuration
ui-install    -> package.json + bun.lock
ui-build      -> UI source/config + public assets + bun.lock
ui-stage      -> ui-build output + export verification + content fingerprint
artifact      -> fresh ui-stage + build-go-stack backend artifact
```

Do not make the normal backend test or build depend on `ui-build`. The complete `artifact` target must prove that the staged export matches current UI inputs.

## Preserve caches and content identity

- Preserve Bun's download cache and `.next/cache` locally and in CI.
- Install a worktree-local `node_modules`; do not symlink it outside the worktree.
- Fingerprint UI sources, `public/`, Next/Tailwind/TypeScript configuration, `components.json`, shadcn primitives, `package.json`, and `bun.lock`.
- Exclude `.next`, `out`, `node_modules`, logs, and unrelated backend files from the fingerprint.
- Build into a temporary narrow staging directory, validate it, then replace the generated embed directory safely.
- Do not overwrite staged files when contents are identical; timestamp churn invalidates the Go embed package and forces relinking.
- Update the fingerprint only after build, copy, and export validation succeed.

Next.js recommends persisting `.next/cache` in CI. See [Next.js CI build caching](https://nextjs.org/docs/pages/guides/ci-build-caching).

## Match checks to risk

1. Run Oxlint during iteration.
2. Run TypeScript when types or contracts changed and a build is not already planned.
3. Add or run React Testing Library only for high-risk interaction state that lint and types cannot falsify.
4. Run the production export when frontend inputs changed or before staging.
5. Run the combined artifact gate when the Go embedding seam changed or before release.
6. Use scripted Playwright only at the final regression gate, when changing its coverage, or when cheaper checks cannot falsify cross-app behavior.

Record warm wall times for standard checks. Measure clean baselines separately and infrequently. Never clear caches in the normal loop merely to produce a timing number.

## Guard staged correctness

- Fail `artifact` when the export or content fingerprint is absent or stale.
- Make backend-only builds identify the current staged UI and warn when stale without silently rebuilding it.
- Verify CSS links, font URLs, manifest and icon presence before replacing staging.
- Never cache secrets or allow undeclared frontend environment values to affect output.
- Provide an explicit diagnostic clean target without making it a routine dependency.
