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

# Refresh the index cache size when a scan ends

## Purpose

The settings window's `Cache` tab shows `Index cache: <size>`, fetched by the
`index_size` command only when the window opens and after Clear Cache. A scan
that runs while the window is open fills the cache without the figure moving,
so the number the user reads is stale exactly when it changes most. Refreshing
it once per scan end, from the `scan-state` listener the window already has,
keeps the figure honest with no polling.

## Steps

- [x] Step 1: Re-fetch `index_size` in the settings window when `scan-state` turns false
  - Done when:
    - With the settings window open on the `Cache` tab, opening a cold folder
      in the main window updates `Index cache: <size>` when the scan finishes,
      and every later scan end updates it again.
    - While a clear is in flight (`clearInFlight`), the "Clearing the index
      cache…" text is never replaced by the scan-end refresh; the Clear Cache
      path's own `index_size` fetches remain the ones that put the figure back.
    - No refresh runs during a scan (only on the end transition) and no timer
      or interval is added.
    - `mise run ci` passes.
    - `docs/usage.md`'s Clear Cache paragraph says the size refreshes when a
      scan ends.
  - Implementation approach:
    - `crates/app/ui/src/settings.ts`, the `scan-state` listener: remember the
      previous `scanRunning`, and when it was true and `payload` is false and
      `!clearInFlight`, `invoke<string>("index_size")` and pass the result to
      `showIndexSize`. Guard on the true->false transition rather than on every
      `false` payload, since the backend can emit `false` more than once in a
      row.
    - Guard `clearInFlight` again inside the `.then` before calling
      `showIndexSize`, so a clear that starts while the fetch is in flight
      keeps its "Clearing…" text. Keep it to a few lines.
    - The post-clear rescan is covered by the same code; no separate handler.
    - No frontend test: `settings.ts` is a DOM-bound entry module with no
      exports; do not extract code just to test it.
    - `docs/usage.md`: add one clause saying the size figure refreshes each
      time a scan ends. Extend `todo.md`'s Clear Cache manual check (4) with
      "and the size figure updates when the scan ends".
    - Manual check (record in the PR body, run by the user after merge): open
      Settings > Cache, open a folder not scanned before (or Clear Cache
      first) with the settings window visible, and confirm the size changes
      when the main window's scan finishes.

## Trade-offs and risks

- Transition vs. every-false refresh: the transition avoids redundant calls;
  an extra call would only cost one file-size read.
- Settings opened after a scan already ended: the initial `index_size` fetch
  already shows the fresh figure.

## Progress

- 2026-09-22: Step 1 landed — the settings window's `scan-state` listener
  re-fetches `index_size` on the true->false transition (guarded by
  `clearInFlight`), and `docs/usage.md` / `todo.md` were updated accordingly.
- (2026-09-22) Step 1 complete
