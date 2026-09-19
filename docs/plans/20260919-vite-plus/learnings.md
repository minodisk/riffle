# Learnings

## Step 1

- main gained `crates/app/ui/settings.html` (settings window, opened via
  `WebviewUrl::App("settings.html")`). It is a second Vite input under
  `build.rollupOptions.input`; its script now points at `./src/settings.ts`.
  `dist/` contains `index.html` and `settings.html` at the root, so the Rust
  URL is unchanged.
- The TypeScript-Go checker in `vp check` ships a newer DOM lib where
  `HTMLElement.hidden` is `boolean | "until-found"`. `tsc` 5 accepted
  `setFilterMenuOpen(filterMenu.hidden)`; tsgo rejects it (TS2345). Fixed with
  `!!filterMenu.hidden` (same truthiness, one-line change in `main.ts`).
- `vp check` also type-checks the root `vite.config.ts`; `process` and
  `node:path` need `@types/node`, added as a devDependency (`^24`, matching the
  Node pin). The plan did not anticipate this dependency.
- `vp check --no-fmt` warnings (do not fail): `no-useless-empty-export` in
  `main.ts` (Step 2) and `no-unused-vars` on `LOCAL_PATH` in
  `.claude/skills/merge-settings/scripts/merge.js` (not in the plan; Step 2
  should either fix it or scope the lint).
- `minify` is a boolean (`!TAURI_ENV_DEBUG`) rather than Tauri's guide's
  `"esbuild"`: Vite+ builds with Rolldown/Oxc, so the esbuild minifier name
  was avoided.
- Debug `cargo check`/`clippy`/`test` of `riffle-app` succeed with no
  `ui/dist` present (verified by deleting it), so the `test` task needs no
  `vp build`.
- `beforeBuildCommand` (`pnpm vp build`) runs from the repo root: `mise run
  tauri:release` built `dist/` and bundled `Riffle.app`. The DMG step then
  failed in `bundle_dmg.sh` under the agent sandbox (hdiutil); not related to
  the frontend.
- Built `dist/*.html` reference only `/assets/...` files.
- Not verified by the agent (manual): `tauri:dev` HMR (`style.css` hot update,
  `main.ts` full reload expected), the feature walkthrough, and running the
  bundled app offline. Nothing Node 24 specific surfaced.

## Step 2

- Removed the trailing `export {};` (and the blank line before it) from
  `main.ts`.
- The `LOCAL_PATH` `no-unused-vars` warning in
  `.claude/skills/merge-settings/scripts/merge.js` was resolved by adding
  `.claude/**` to `lint.ignorePatterns` rather than editing the script: lint
  scope follows the Oxfmt scope decision (frontend + JS tooling at the root),
  and the agent skill scripts are outside it. The deferred item from Step 1 is
  handled.
- `pnpm exec vp check --no-fmt`: "Found no warnings, lint errors, or type
  errors in 7 files".

## Deferred issues (todo candidates)

None.
