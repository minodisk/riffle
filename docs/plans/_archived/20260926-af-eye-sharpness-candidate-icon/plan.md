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

# AF eye sharpness label, no Focus row, and a strip candidate icon

## Purpose

The focus candidate cue currently surfaces in the meta pane as two Analysis
rows (`Focus`: `Candidate` / `Not a candidate`, and `Eye sharpness`) and on
the viewer's focus mark color. The `Focus` row duplicates what the mark color
and the `Focus candidates` filter already say, and `Eye sharpness` does not
say which eyes it measures. This work renames the label to `AF eye sharpness`
(the eyes of the face nearest the AF point), drops the `Focus` row, and marks
each candidate directly on the strip with a green Lucide `scan-face` icon in
the cell's bottom-left corner, so candidates can be spotted while scrolling
the strip without opening the filter. The icon is inline SVG (no npm
dependency) and ships with Lucide's license notice.

Code identifiers (`FocusCandidate`, `candidate`, `eye_sharpness`, the SQLite
column) do not change; the filter menu, `FOCUS_MARK_COLORS`, and the MCP
companion are untouched.

## Steps

- [x] Step 1: Rename the label, drop the `Focus` row, add the strip candidate icon, and record the Lucide notice
  - Done when:
    - `metaGroups` in `crates/app/ui/src/meta.ts` emits `AF eye sharpness`
      (instead of `Eye sharpness`) and no `Focus` row; `candidateLabel` and
      the now-unused parts of the header comment are removed; `FocusCue` /
      `FocusCandidate` stay. `crates/app/ui/src/meta.test.ts` is updated
      (the three tests at ~lines 167–205 that expect `Focus` rows / the
      `Eye sharpness` label) and `pnpm exec vp test` passes.
    - A strip cell shows a green Lucide `scan-face` icon at the bottom-left of
      the image box, above the file name, only while the file's
      `focus.candidate === "candidate"`; nothing for `not_candidate` /
      `unknown`. It appears live as `faces-progress` streams, on a folder
      re-open (`refreshEntries`), and survives `refilter` (which calls
      `strip.setFiles` and clears the strip's per-index stores).
    - Lucide's LICENSE text (ISC + the Feather MIT section) is in the repo
      verbatim, a source comment next to the SVG names Lucide, `scan-face`,
      the version it was taken from (lucide-static v1.48.0) and points at
      the notice file, and the notice reaches the built frontend (see the
      approach; the chosen mechanism and its verification are recorded in
      `learnings.md`).
    - `README.md`, `README.ja.md`, `docs/usage.md` and `CLAUDE.md` no longer
      describe a `Focus` row or an `Eye sharpness` label, mention the strip
      icon, and credit Lucide the way YuNet is credited.
    - `mise run ci` passes.
  - Implementation approach:
    - **Meta pane** (`crates/app/ui/src/meta.ts`): in the Analysis section,
      keep `["Sharpness", ...]` and change the eye row to
      `["AF eye sharpness", focus?.eye_sharpness?.toFixed(1) ?? null]`; delete
      the `["Focus", candidateLabel(...)]` row and the `candidateLabel`
      function. `metaGroups`'s signature stays (`focus?: FocusCue | null`)
      since `main.ts` passes the whole `Focus`. Update the comment above
      `metaGroups` ("The focus candidate state gets a `Focus` row ...").
      Tests: `shows the focus candidate state and the eye sharpness after
      Sharpness` becomes a test that the AF eye sharpness follows `Sharpness`
      for both `candidate` and `not_candidate`; `leaves out both rows when
      the state is unknown with no value` becomes "leaves out the row when
      there is no value"; `shows the analysis group for the eye sharpness
      alone` drops the `Focus` row from its expectation.
    - **Strip icon, per-index store** (`crates/app/ui/src/strip.ts`): follow
      the existing pattern of `sharpness` / `bursts` maps + `setSharpness` /
      `setBurst` + `paintSharpness` / `paintBurst`:
      - add `candidates: Set<number>` (indices whose file is a candidate),
        `export function setCandidate(index: number, candidate: boolean)`,
        `paintCandidate(index, cell)` (toggles `cell.candidate.hidden`), call
        `paintCandidate` from `createCell`, and clear the set in `setFiles`
        next to `sharpness.clear()`.
      - add `candidate: HTMLSpanElement` to the `Cell` interface; in
        `createCell` create `<span class="candidate">` whose content is the
        inline SVG. Build it either with `innerHTML` from a module-level
        constant string or via `createElementNS`; a constant string
        (`const SCAN_FACE_SVG = \`<svg ...>...</svg>\``) is the simplest and
        keeps the license comment next to the markup. The SVG is exactly
        lucide-static v1.48.0's `icons/scan-face.svg` body:
        `viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"`
        with paths `M3 7V5a2 2 0 0 1 2-2h2`, `M17 3h2a2 2 0 0 1 2 2v2`,
        `M21 17v2a2 2 0 0 1-2 2h-2`, `M7 21H5a2 2 0 0 1-2-2v-2`,
        `M8 14s1.5 2 4 2 4-2 4-2`, `M9 9h.01`, `M15 9h.01` (drop the
        `class="lucide lucide-scan-face"` attribute and the `width`/`height`
        attributes; size it in CSS). Set `aria-hidden="true"`.
      - color: set `cell.candidate.style.color = FOCUS_MARK_COLORS.candidate`
        (import from `./focus.js`, which is DOM-free) so the icon and the
        viewer mark share the one constant instead of a second `#3f3` in CSS;
        the SVG's `stroke="currentColor"` picks it up.
    - **CSS** (`crates/app/ui/style.css`): add `.cell span.candidate` next to
      `.cell span.count`, mirroring it on the left: `bottom: 26px; left: 2px;
      right: auto; width: auto; padding: 0; line-height: 0;
      filter: drop-shadow(0 0 2px #000);` (a `text-shadow` does not apply to
      SVG strokes; `drop-shadow` gives the same dark halo the other badges
      have), `.cell span.candidate svg { width: 12px; height: 12px;
      display: block; }`, and `.cell span.candidate[hidden] { display: none; }`
      as `.cell span.sharpness[hidden]` does (the base `.cell span` rule sets
      `position: absolute`, which would otherwise override `hidden`). The
      sharpness bar occupies `top: 24px; height: 96px` (its bottom at 120px
      from the cell top); a 12px icon at `bottom: 26px` in a 168px cell
      starts at 130px, so it is clear vertically. Verify in the running app
      that the icon does not collide with the bar or the burst count on a
      current cell (`n/m` text at bottom-right) and sits above the name span;
      if the 2px left edge reads as touching the bar's column, bump `left`
      to 6px and note it.
    - **Feeding the strip** (`crates/app/ui/src/main.ts`): add
      `applyCandidates()` beside `applySharpness()` /`applyBursts()`:
      `files.forEach((path, at) => strip.setCandidate(at, entries.get(path)?.focus?.candidate === "candidate"))`.
      Call it (1) in `refreshEntries` after `applyBursts()`, (2) inside
      `refilter` after `strip.setFiles(...)` where `applySharpness` /
      `applyBursts` are re-run (the strip's per-index store is cleared there;
      see `docs/agents/tauri-app.md` "A derived-state refresh has to run even
      when `refilter` short-circuits"), and (3) in the `faces-progress`
      listener (~line 2164) right after `applyFaceReady(...)`, before the
      `shownCandidates` refilter. Calling it over all `files` per batch is
      O(files) and matches `applySharpness`; a per-path `setCandidate`
      using the path's index is an acceptable cheaper variant if the
      implementer prefers, but keep it simple. Also check `faces-done` ->
      `refreshEntries` covers the final state (it does via (1)).
    - **Lucide notice**: the precedent is `crates/core/models/LICENSE`
      (YuNet's MIT text next to the asset, credited in `README.md` /
      `README.ja.md` under "Offline face detection"; it is not shipped in
      the bundle, `tauri.conf.json` has no `bundle.resources`, and the model
      is `include_bytes!`ed). Mirror it: add `crates/app/ui/LICENSE-lucide`
      holding https://github.com/lucide-icons/lucide/blob/main/LICENSE
      verbatim (`ISC License / Copyright (c) 2026 Lucide Icons and
      Contributors`, the `---` separator, the Feather icon list, and the MIT
      text `Copyright (c) 2013-present Cole Bemis`). Next to `SCAN_FACE_SVG`
      put a `/*! ... */` comment: "Lucide `scan-face` icon, lucide-static
      v1.48.0 (https://github.com/lucide-icons/lucide), ISC License,
      Copyright (c) 2026 Lucide Icons and Contributors; full notice in
      `crates/app/ui/LICENSE-lucide`" — with the ISC permission sentence
      included, so the comment itself satisfies ISC. Use the `/*!` form so
      the minifier can keep it as a legal comment.
    - **Shipping the notice in the built frontend without touching
      `tauri.conf.json`**: run `pnpm exec vp build` and grep `ui/dist` for
      `Lucide`. If the `/*! */` comment survives the Vite+ (rolldown/oxc)
      minifier by default, nothing more is needed; record that in
      `learnings.md`. If it is stripped, set the legal-comments option in
      the root `vite.config.ts` build section (check the Vite+ / rolldown
      `legalComments` option name; `vite.config.ts` is not release-shaping)
      and re-verify. Only if neither works fall back to
      `bundle.resources` in `tauri.conf.json`, and call it out in the PR.
    - **Docs**: `README.md` line ~81 ("the meta pane shows the state as a
      `Focus` row and the eye sharpness beside it") -> the meta pane shows
      the `AF eye sharpness`, and the strip marks each candidate with a
      green face icon at the cell's bottom-left; add a Lucide credit
      ("[Lucide](https://lucide.dev) `scan-face` icon, ISC license") near
      the YuNet credit. Mirror both in `README.ja.md` line ~49 / ~51 (Japanese
      body, same PR). `docs/usage.md` lines ~113–118: Analysis holds the
      sharpness score and the `AF eye sharpness` (left out when there is
      none); drop the `Focus` row sentence; mention the strip icon in the
      focus mark bullet (~line 52). `CLAUDE.md` line ~50: "Analysis, whose
      rows include the focus candidate state and the eye sharpness" -> "the
      AF eye sharpness"; optionally note the strip's candidate icon and the
      Lucide notice file. Leave `docs/performance.md` and `crates/core`
      comments alone (prose about the measurement, not the label). `lychee`
      checks Markdown links, so a link to lucide.dev / GitHub must resolve.
    - Commit as `feat(app): ...` (Conventional Commits, English).

## Trade-offs and risks

- **Single step vs. two PRs.** The meta pane change and the strip icon are
  separable, but planned as one because the user asked for one feature and
  the docs sentences describe both in the same bullet.
- **Where the Lucide text lives.** `crates/app/ui/LICENSE-lucide`, mirroring
  `crates/core/models/LICENSE` (approved by the user). A repo-level
  `THIRD_PARTY_NOTICES.md` covering YuNet too is out of scope.
- **Shipping the notice.** Relies on a preserved `/*!` legal comment in the
  built JS, verified by grepping `ui/dist`. Fallbacks in order: a
  `vite.config.ts` legal-comments option, then `bundle.resources` in
  `tauri.conf.json` (release-shaping; avoid unless the first two fail).
- **Color source.** The icon color comes from `FOCUS_MARK_COLORS.candidate`
  via inline style, to avoid a second `#3f3` in CSS.
- **Shadow style.** SVG needs `filter: drop-shadow(0 0 2px #000)` instead of
  `text-shadow`; check visually.
- **No strip unit test.** `strip.ts` has no tests (vitest runs in `node`), so
  the icon is verified manually; the meta pane change is covered by
  `meta.test.ts`.
- `applyCandidates()` over all `files` on every `faces-progress` batch is
  O(files), the same cost class as `applySharpness`.

## Progress

- 2026-09-26: Step 1 done: meta pane label renamed to `AF eye sharpness` and
  `Focus` row dropped, strip `scan-face` candidate icon added, Lucide notice
  kept in the minified JS through `build.rolldownOptions.output.comments.legal`
  (see learnings.md)
