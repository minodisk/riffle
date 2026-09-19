<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Migrate the frontend toolchain to Vite+

## Purpose

The frontend under `crates/app/ui` is plain HTML/CSS/TS compiled by `tsc`
alone: no bundler, no linter, no formatter, no test runner. Tauri serves the
`ui/` directory as-is, with `tsc --watch` emitting `.js` files next to the
sources. Every check the frontend gets today is `tsc --noEmit`, and there is
nowhere to put a unit test for the pure logic accumulating in the 1500-line
`main.ts`.

Vite+ (`vite-plus`, VoidZero's `vp` CLI) bundles Vite, Oxlint, Oxfmt, Vitest
and a TypeScript-Go type checker behind one binary and one `vite.config.ts`.
Adopting it in one go gives the app a real dev server with HMR, a production
bundle under Tauri's `frontendDist`, a linter, a formatter, a type checker and
a test runner, all wired into `mise run ci` and therefore into the existing
GitHub Actions workflows.

Research findings that shape the plan (2026-09-19):

- `vite-plus` 0.3.3 on npm is **MIT-licensed** (the package's `LICENSE`, the
  site's "Free and open source under the MIT license", and the July 2026 beta
  announcement's "fully open source under the MIT license"). It is in
  **beta**: "stable, but not yet complete"; migrations "may still need manual
  follow-up".
- Engines: `node ^20.19.0 || ^22.18.0 || >=24.11.0`. The repo moves to Node
  24 (decided) so the next Vite major cannot force it mid-way.
- Install is an ordinary devDependency; the `vp` bin runs through
  `pnpm exec vp`. The curl installer and the `voidzero-dev/setup-vp` action
  are not needed because mise already provides Node and pnpm.
- `vp` subcommands used: `dev`, `build`, `check` (fmt + lint + type check),
  `fmt`, `lint`, `test`. Configuration lives only in the root
  `vite.config.ts` (`lint`, `fmt`, `test` blocks via `defineConfig` from
  `vite-plus`); the docs discourage `.oxlintrc.json` / `.oxfmtrc`.
- Verified in a scratchpad copy of the UI: Vite resolves
  `new URL("./worker.js", import.meta.url)` to `worker.ts` and emits it as its
  own chunk, so the worker needs no code change; `vp fmt` honours
  `.gitignore`; `test.include` is relative to Vite's `root`; `vp check` with
  `lint.options.typeCheck: true` type-checks through tsgolint **without the
  `typescript` package installed**, given a root `tsconfig.json` whose
  project references point at `crates/app/ui`.

Decisions taken before execution: `vp check` is the single static check
(`tsc` and `typescript@5` are removed); Oxfmt owns only the frontend and the
JS tooling files; Oxfmt's default `printWidth` (100) is kept; `vp test` runs
in the `test` mise task on all three CI platforms; Node is bumped to 24.21.0
in Step 1.

## Steps

- [x] Step 1: Build and serve the frontend through Vite, on Node 24
  - Done when:
    - `mise.toml` pins `node = "24.21.0"` (latest 24.x; satisfies
      `vite-plus`'s `>=24.11.0`). **CI-wide impact**: every job in `ci.yml`
      and `release.yml` resolves Node through `jdx/mise-action`, so all of
      them switch runtime in this PR; the PR description calls this out.
    - `pnpm install --frozen-lockfile` installs `vite-plus` pinned to an
      exact version (`0.3.3` at planning time); `typescript` is removed from
      `devDependencies` (nothing else in the repo depends on it: only
      `package.json`, `mise.toml` and `tauri.conf.json` reference `tsc`, all
      changed here, plus `docs/agents/tauri-app.md`, updated in Step 6).
    - `mise run tauri:dev` opens the app served by the Vite dev server (HMR
      works: editing `style.css` updates the running app) and every existing
      feature that goes through `window.__TAURI__` and the decode worker still
      works (open folder, filmstrip, 1:1 zoom, ratings, settings window,
      update check).
    - `mise run tauri:release` bundles from `crates/app/ui/dist` and the bundled
      app runs offline (the built `dist/index.html` references only local
      `/assets/...` files; no network requests for assets).
    - `mise run ci` passes with the `lint` task's `tsc --noEmit` line replaced
      by `pnpm exec vp check --no-fmt` (the `lint` and `fmt` blocks are
      configured in Steps 2-3; here `vp check --no-fmt` runs Oxlint's default
      rules plus the type check). If the default Oxlint rules flag anything,
      note it and, if it is only the known `export {}` warning, leave the fix
      to Step 2 (warnings do not fail `vp check`).
    - No `crates/app/ui/**/*.js` build products are produced or tracked.
  - Implementation approach:
    - Root `vite.config.ts` (next to `package.json`), `import { defineConfig }
      from "vite-plus"`, with `root: "crates/app/ui"`, `clearScreen: false`,
      `server: { port: 1420, strictPort: true }`, `build: { outDir: "dist",
      emptyOutDir: true }`, `lint: { options: { typeAware: true, typeCheck:
      true } }`. Follow Tauri's Vite guide for the rest: `envPrefix:
      ["VITE_", "TAURI_ENV_*"]`, `build.target` of `chrome105` on Windows /
      `safari13` elsewhere via `process.env.TAURI_ENV_PLATFORM`,
      `build.sourcemap` / `minify` keyed on `TAURI_ENV_DEBUG`. Add `crates/`
      Rust sources to `server.watch.ignored` so a Rust rebuild does not
      trigger a reload storm.
    - `crates/app/tauri.conf.json` (**release-shaping, call out in the PR**):
      `build.devUrl: "http://localhost:1420"`, `build.frontendDist:
      "ui/dist"` (relative to the `tauri.conf.json` directory, see
      `docs/agents/tauri-app.md`), `beforeDevCommand: "pnpm vp dev"`,
      `beforeBuildCommand: "pnpm vp build"`. Add `"vp": "vp"` to
      `package.json` scripts so `pnpm vp` works like `pnpm tauri`. Verify the
      before-commands run with the repo root as cwd (the current `tsc -p
      crates/app/ui/tsconfig.json` relies on that); if not, use Tauri's object
      form `{ "script": ..., "cwd": ... }`.
    - `crates/app/ui/index.html` (and any other HTML entry, e.g. a settings
      window page, which then needs `build.rollupOptions.input`):
      `<script type="module" src="./src/main.ts">` (Vite consumes the TS
      source; today it points at the emitted `main.js`). Keep `.js`-suffixed
      imports and the `./worker.js` URL as they are: Vite maps them to the
      `.ts` sources (verified) and `moduleResolution: bundler` accepts them.
    - `.gitignore`: replace `crates/app/ui/**/*.js` with `crates/app/ui/dist`.
    - Root `tsconfig.json` with `"files": [], "references": [{ "path":
      "./crates/app/ui" }]` so tsgolint and editors find the UI project (this
      layout was the one verified in the scratchpad); add `"composite": true`
      to `crates/app/ui/tsconfig.json` if references require it. Editors keep
      working from the same tsconfigs with their own bundled TypeScript.
    - `tauri.d.ts` stays as the global `window.__TAURI__` declaration;
      `withGlobalTauri: true` is unchanged, so the injected global still exists
      under the Vite dev server (the webview injects it, not the page).
      `csp: null` stays.
    - Do not split or refactor `main.ts`.
    - Check that the CI `test` matrix, which compiles `crates/app` (and thus
      `tauri::generate_context!()` embedding `frontendDist`), does not need a
      prior `vp build` for `cargo clippy`/`cargo test`; if it does, add
      `pnpm install --frozen-lockfile` + `pnpm exec vp build` to the `test`
      task now (Step 5 adds the pnpm install there anyway).
    - Record in `learnings.md` whether HMR works for `main.ts` (a full reload
      is expected and acceptable: the module has top-level side effects) and
      anything Node 24 changed.

- [x] Step 2: Configure Oxlint (`lint` block) and fix the one existing finding
  - Done when: the `lint` block in `vite.config.ts` has `ignorePatterns:
    ["crates/app/ui/dist/**", ".claude/**"]` (`.claude/**` excludes agent
    skill scripts, which are outside the Oxfmt/lint scope of frontend + JS
    tooling at the root) alongside the type-check options from Step 1;
    `pnpm exec vp check --no-fmt` exits 0 with no warnings on the current
    sources; `mise run ci` passes; CI's `lint` job is green.
  - Implementation approach:
    - Keep Oxlint's default category (`correctness`) plus the default plugins;
      do not enable extra plugins or categories in this step.
    - The only finding at planning time is `typescript(no-useless-empty-export)`
      on the trailing `export {};` in `crates/app/ui/src/main.ts`. `main.ts`
      already has a top-level `import`, so the line is redundant; remove it
      (one-line change, no other edit to `main.ts`). Fix any other finding
      that has appeared since with the smallest change.
    - `.github/workflows/ci.yml` needs no change: the `lint` job already runs
      `mise run lint` with pnpm installed and cached.

- [x] Step 3: Configure Oxfmt (`fmt` block), no reformat
  - Done when: `vite.config.ts` has a `fmt` block; `pnpm exec vp fmt --check`
    lists only files under `crates/app/ui` plus the JS tooling files; nothing
    is reformatted in this step and `mise run ci` still passes (the `lint`
    task keeps `--no-fmt` until Step 4).
  - Implementation approach:
    - Scope (decided: frontend + JS tooling only). Measured on the real repo,
      an unscoped `vp fmt --check .` flags 57 files because Oxfmt also
      formats Markdown, JSON, YAML and TOML. `fmt.ignorePatterns` therefore
      covers `**/*.md` (README, todo, CHANGELOG owned by release-please,
      every `docs/**` plan and review), `.release-please-manifest.json`,
      `release-please-config.json`, `crates/app/capabilities/**`,
      `crates/app/tauri.conf.json`, `target/**`, `crates/app/ui/dist/**`.
      What remains: `crates/app/ui/**`, `package.json`, `vite.config.ts`,
      the `tsconfig.json`s. Confirm the list with `vp fmt --check` and record
      it in `learnings.md`.
    - `printWidth`: keep Oxfmt's default (100); no override in the config.
    - `mise.toml` `fmt` stays `cargo fmt --all` in this step: pr-runner runs
      `mise run fmt` before committing, so adding `pnpm exec vp fmt` here
      would reformat the 6 frontend files and break "no reformat". The task
      change moves to Step 4.

- [ ] Step 4: Reformat the frontend with Oxfmt and enforce the format check
  - Done when: the PR contains **exactly two commits**: (a) a formatting-only
    commit produced by `pnpm exec vp fmt` with no hand edits
    (`style(ui): format with oxfmt`), and (b) a commit that makes the `fmt`
    task run `cargo fmt --all` then `pnpm exec vp fmt` and drops `--no-fmt`
    from the `lint` task so `pnpm exec vp check` runs format, lint and type
    check together; `mise run ci` passes; `mise run tauri:dev` still runs.
  - Implementation approach:
    - Depends on Step 3 being merged so the reformat uses the agreed config.
    - Expected size at the default width (measured on a copy at planning
      time): ~330 diff lines across `main.ts`, `index.html` (attribute
      splitting of the long `<button>`/`<svg>` tags), `strip.ts`,
      `tauri.d.ts`, `worker.ts`; `style.css` 0.
    - PR description: review the formatting commit with a whitespace-
      insensitive diff; the second commit is the only one with semantic
      content.
    - Do not fold any other change (lint fixes, refactors) into this PR.
    - Other in-flight UI branches will conflict with this reformat; they
      rebase and run `mise run fmt` afterwards.

- [ ] Step 5: Run Vitest via `vp test` with a real test on pure logic
  - Done when: at least one `*.test.ts` under `crates/app/ui/src` exercises a
    pure function and passes; `pnpm exec vp test` runs once (not watch) and
    exits 0; `mise run test` runs it; CI's `test` job is green on all three
    OSes with the pnpm store cached.
  - Implementation approach:
    - `test` block in `vite.config.ts`: `include: ["src/**/*.test.ts"]`
      (relative to `root`, verified), `environment: "node"`. `vitest` is a
      dependency of `vite-plus`; if the type check cannot resolve the
      `"vitest"` import from the test file, add `vitest` as an explicit
      devDependency at the version `vite-plus` bundles (0.3.3 bundles Vitest
      4.1.11) rather than widening `types`.
    - Candidate pure functions in `main.ts`: `focalRange(mm)`,
      `exifKey(exif, group)`, `baseName(path)`. Move `focalRange` + `exifKey`
      (together they define the EXIF filter-menu grouping) into
      `crates/app/ui/src/exif.ts` and import them from `main.ts`; that is the
      only edit to `main.ts`. Test `exifKey` with hand-built `Exif` objects
      (each group's label/order) and `focalRange` at the bucket boundaries.
      Do not extract anything that touches `document`/`window`.
    - `mise.toml` `test`: `pnpm install --frozen-lockfile` then
      `pnpm exec vp test`, alongside `cargo clippy` / `cargo test`.
    - `.github/workflows/ci.yml` (**release-shaping, call out**): add the
      "Resolve the pnpm store path" + `actions/cache` steps to the `test`
      job, copied from the `lint` job, with `shell: bash` on the resolve step
      as `release.yml` does so it works on the Windows runner.
    - The UI `tsconfig.json`'s `include: ["src/**/*.ts"]` already covers
      `*.test.ts`; the production build does not pull tests in because
      nothing imports them from `main.ts`.

- [ ] Step 6: Update the documentation
  - Done when: `docs/agents/tauri-app.md`, `CONTRIBUTING.md` and the
    `CLAUDE.md` layout section describe the Vite+ setup; `mise run ci` passes.
  - Implementation approach:
    - `CLAUDE.md` layout: replace "TypeScript compiled by `tsc` only, no
      bundler" with the Vite+ description: root `vite.config.ts`, `pnpm exec
      vp {dev,build,check,fmt,test}`, checked by `mise run ci`.
    - `CONTRIBUTING.md` "Building and running": `tauri:dev` starts the Vite
      dev server on `localhost:1420` through `beforeDevCommand`; `mise run
      fmt` formats both Rust and the frontend; `mise run lint` runs
      `vp check` (format, lint, types); frontend tests via `pnpm exec vp test`
      (`vp test watch` while developing); Node 24 comes from `mise install`.
    - `docs/agents/tauri-app.md`: update the `frontendDist` item to `ui/dist`
      and any `tsc --noEmit` mention to `vp check`; add a short "Frontend"
      section with the items learned in Steps 1-5 (tagged
      Hit/Measured/Inferred per the file's convention): `.js`-suffixed imports
      resolve to `.ts` under Vite, `vp fmt` honours `.gitignore`,
      `test.include` is root-relative, the Oxfmt scope decision, anything that
      broke.
    - Do not touch `README.md` (user-facing only).

## Trade-offs and risks

- **Beta toolchain.** Vite+ is MIT but explicitly beta; a `vp` release can
  change CLI flags or config keys. Mitigation: pin `vite-plus` exactly (no
  caret) and bump deliberately. Fallback if the beta proves unstable: plain
  `vite` + `oxlint` + `oxfmt` + `vitest` as separate devDependencies with the
  same `vite.config.ts` shape, which changes only the mise tasks. With
  `typescript@5` removed, that fallback would need a type checker again
  (`tsc` or `tsgo`).
- **Single type checker inside `vite-plus`.** Type checking now follows
  whatever TypeScript-Go version `vite-plus` bundles; a `vite-plus` bump can
  change type-check results. Editors keep using their own TypeScript against
  the same tsconfigs, so an editor and `vp check` can occasionally disagree.
- **Node 24 bump is CI-wide.** Every `ci.yml` and `release.yml` job picks up
  Node 24.21.0 from `mise.toml` in the Step 1 PR; the release workflow's
  `tauri-action` runs `pnpm tauri build` on it. Watch the first release
  build after merging.
- **`tauri.conf.json` is release-shaping.** The `build` block change is
  exercised by `release.yml` through `tauri-action` -> `pnpm tauri build` ->
  `beforeBuildCommand` -> `pnpm vp build`. The release job already runs
  `pnpm install --frozen-lockfile`, so no workflow edit is needed, but a
  broken `vp build` on Windows or macOS-Intel would only surface at release
  time. Mitigation: `mise run tauri:release` locally in Step 1, and the
  `generate_context!` check described in Step 1.
- **Offline requirement.** Vite's production bundle inlines everything under
  `dist/assets`; nothing is fetched from a CDN. The dev server is
  `localhost` only. `csp: null` stays, so hashed asset names need no CSP work.
- **HMR and `main.ts` side effects.** `main.ts` registers Tauri event
  listeners at module top level; Vite HMR full-reloads it rather than
  hot-swapping, which re-registers listeners on a fresh page (fine). No
  `import.meta.hot` handling is planned.
- **`index.html` formatting.** Oxfmt splits the long `<button>`/`<svg>` tags
  one attribute per line. If that turns out unwanted in Step 4 review, add
  `crates/app/ui/index.html` to `fmt.ignorePatterns` in the same PR's second
  commit.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-19) Step 2 complete
