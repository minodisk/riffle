# Learnings

## Step 1

- The Lucide LICENSE (raw `main`) and lucide-static v1.48.0's
  `icons/scan-face.svg` were fetched; the seven paths match the plan exactly.
  `crates/app/ui/LICENSE-lucide` is the raw LICENSE byte for byte.
- **The `/*!` notice does not survive the default build.** Vite 8 (and the
  vite-plus-core it is aliased to) sets rolldown's `output.comments` to
  `{ annotation: !minify, jsdoc: !minify, legal: !minify }`, so a minified
  build drops legal comments. Fix: `build.rolldownOptions.output.comments`
  in the root `vite.config.ts` with `legal: true`. An explicit `comments`
  object replaces Vite's defaults rather than merging into them in practice
  (with `{ legal: true }` alone the minified bundle kept 42 `@__PURE__` and a
  `@vite-ignore` comment, ~700 bytes), so `annotation` / `jsdoc` are restated
  as `!!process.env.TAURI_ENV_DEBUG`, which is what Vite derives from
  `minify: !process.env.TAURI_ENV_DEBUG`. Verified with `pnpm exec vp build`:
  `crates/app/ui/dist/assets/index-*.js` holds the full `/*! Lucide
  scan-face ... */` block (ISC permission sentence included) and no
  annotation comments. `tauri.conf.json` was not touched.
- `pnpm` is only on the PATH through mise in this environment
  (`mise exec -- pnpm ...`), and a fresh worktree needs
  `pnpm install --frozen-lockfile` before `vp build`.
- The icon's placement (clear of the sharpness bar and the burst count, above
  the name) was worked out from the CSS geometry only; the running app was not
  inspected from this agent, so check it visually before merging.
