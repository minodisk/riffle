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

## Step 3

- `vp fmt --check` with the `fmt.ignorePatterns` scope checks 21 files and flags 6:
  `crates/app/ui/index.html`, `src/main.ts`, `src/settings.ts`, `src/strip.ts`,
  `src/tauri.d.ts`, `src/worker.ts`. The JS tooling files (`package.json`,
  `vite.config.ts`, the `tsconfig.json`s) are already clean. `settings.ts` was
  not in the planning-time estimate for Step 4.
- Adding `pnpm exec vp fmt` to the `fmt` mise task in this step would make
  pr-runner's pre-commit `mise run fmt` reformat those 6 files, so the task
  change moved to Step 4 (plan updated).

## Step 4

- `pnpm exec vp fmt` touched 6 files (164+/174-): `index.html`, `main.ts`,
  `settings.ts`, `strip.ts`, `tauri.d.ts`, `worker.ts`. `settings.ts` was not in
  the planning-time estimate (it landed on main after the measurement).

## Step 5: Vitest via `vp test`

- `vp test` runs once (no watch) under `mise run test`. At runtime Vitest
  4.1.11 comes through `vite-plus`, but the `vp check` type check failed with
  TS2307 on `import ... from "vitest"` (pnpm does not hoist it), so `vitest`
  was added as an explicit devDependency pinned to 4.1.11, as the plan
  anticipated.
- Extracting `focalRange` + `exifKey` also required moving the `Labelled`,
  `Exif` and `ExifGroup` types and the `focalRanges` table into `exif.ts`
  (the functions depend on them); `main.ts` imports `Exif`, `ExifGroup` and
  `exifKey` back. So the `main.ts` edit is the import plus the removals, not
  the import alone.

## Step 6

- The `tsc`-era frontend notes in `docs/agents/tauri-app.md` were rewritten:
  the `outDir`/`rootDir` pitfall was dropped (nothing emits `.js` any more), and
  the `export {};` advice was replaced since Oxlint now flags it.
