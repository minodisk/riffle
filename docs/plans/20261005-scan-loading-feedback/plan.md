<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Scan loading feedback

## Purpose

Opening a large folder (the todo's case: 500 JPEGs on Windows, 2026-09-28)
shows progress only as the small `scanning N / M` note at the bottom of the
right pane, so it is unclear whether loading has started. The first scan
pass already streams `scan-progress` every 100 ms and the strip already
knows which cells have no thumbnail yet; this work surfaces both: a thin
progress bar over the strip while the first pass runs, and a skeleton pulse
on every strip cell whose thumbnail has not arrived. The faces pass is left
to the status line (a JPEG-only folder has none: `run_scan` stamps JPEG rows
with `FACES_VERSION`, so `faces_todo` is empty). Closes the todo.md section
"App: a large folder gives no visible loading feedback beyond the status
line".

Decisions agreed with the user (2026-10-05): both the bar and the pulse; the
bar covers pass 1 only; it hides immediately on `scan-done`; it shows at 0%
from `openDirectory()` (covering the listing/reconcile phase, accepting a brief
flash on a fully cached reopen); a `resync()` (focus, watch, reload, trash,
restore, rename) shows it only once `scan-progress` reports real work.

## Steps

- [x] Step 1: Add the scan progress bar and the empty-cell pulse
  - Done when:
    - `crates/app/ui/index.html` has `<div id="scan-progress" class="progress" hidden><div class="progress-indicator"></div></div>` as the first child of `#film`, above `#strip-bar`.
    - `crates/app/ui/style.css` gains `.progress` / `.progress-indicator` in the `/* Components */` block (shadcn Progress: track on `var(--muted)`, indicator on `var(--primary)`, `border-radius: 9999px`, `overflow: hidden`; the indicator's width is set by the script) above the `.dialog-box [hidden]` rule, an id rule `#scan-progress:not([hidden])` for height (2 px) and `flex: none`, and `.progress` / `.progress-indicator` rows in the component table of `docs/agents/ui-styling.md`.
    - `style.css` gains a `@keyframes pulse` (shadcn `animate-pulse`: opacity 1 -> 0.5 -> 1, 2 s, `cubic-bezier(0.4, 0, 0.6, 1)`, infinite) applied by `.cell img:not([src])`, and `.cell.failed img:not([src]) { animation: none; }` so a failed cell does not pulse.
    - A new pure module `crates/app/ui/src/progress.ts` exports `progressWidth(done: number, total: number): string` (a `"NN%"` string, `"0%"` when `total` is 0, clamped to `100%`), with `progress.test.ts` covering 0 total, partial, complete, and `done > total`.
    - `crates/app/ui/src/main.ts`: `openDirectory()` un-hides `#scan-progress` at `0%` just before `void startScan(folder)`; the `scan-progress` handler sets the indicator width from `progressWidth(payload.done, payload.total)`; `scan-done` and the `startScan` catch path hide it. Nothing is done on `faces-progress` / `faces-done`.
    - The ui-styling greps (`#[0-9a-f]{3,6}\b|rgba?\(` and `margin|float`) show no new match.
    - `mise run ci` passes.
    - The todo.md section "App: a large folder gives no visible loading feedback beyond the status line" is removed, and a new section "App: the scan loading feedback's manual check is still open" is added with one item: open a folder of 500+ files whose index is cold (or after Clear Cache) on Windows and confirm the bar appears before the first thumbnail, fills, and disappears at the end of the first pass, and that empty cells pulse until their thumbnail lands while failed cells do not.
  - Implementation approach:
    - Follow `docs/agents/ui-styling.md`: colors only through the component classes, size/placement on the id, no margin, `display` on the id scoped `:not([hidden])`.
    - Reuse the existing `scanning` lifecycle in `main.ts` rather than adding a new state flag; the bar's visibility mirrors `scanRunning` for pass 1 only.
    - Keep the status line text unchanged; the bar is additive.
    - The pulse is CSS-only; the virtual list keeps only visible cells, so the animation count is small.
    - Tests are limited to the pure helper (vitest runs in `node`); the DOM wiring and the CSS are covered by the manual-check todo item.

## Trade-offs and risks

- Design: C (bar + pulse) was chosen over A or B alone. B alone is the smallest change (CSS-only, no test) but does not answer "how far"; A alone answers that but leaves the cells looking frozen.
- Pass 2 is not in the bar. Including it means either a second fill cycle on the same bar (confusing) or a second bar, for RAW folders only; the status line's `analyzing N / M` stays as is.
- The bar hides immediately on `scan-done`. A fade would need a `transition` and a `transitionend`/timer guard against a scan that restarts during the fade.
- `scan-progress.total` is the todo count, not the folder size: a mostly cached folder fills the bar in a blink and a fully cached reopen flashes a 0% track between `openDirectory()` and `scan-done`.
- The track uses `--muted` instead of shadcn's `primary/20` to avoid a new translucent fill.

## Progress

- (none yet)
