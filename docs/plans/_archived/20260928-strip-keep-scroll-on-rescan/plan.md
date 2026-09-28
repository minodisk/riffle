<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Keep the strip's scroll and thumbnails across a rescan

## Purpose

Every time files land in the open folder, the strip snaps back to the focused
file and every visible thumbnail reloads. Confirmed by the user on Windows:
open a folder of 25 ARWs, scroll the strip to its end, copy 100 ARWs in; the
watcher-triggered rescan runs in 19 rounds and each round snaps and flickers.
The final display is correct.

Three things cause it, in `crates/app/ui/src/main.ts` and
`crates/app/ui/src/strip.ts`:

1. `refreshEntries` ends in `refilter(anchor, false, hadPendingResume)`:
   `keepScroll` is `false`, so once the new files' capture times arrive and
   the order changes, `strip.setFiles(files, false)` resets `scrollLeft` to
   0. (`resync` already passes `keepScroll = true`.)
2. `refilter`'s tail calls `strip.setCurrent(index)`, and `setCurrent`
   always scrolls the focused cell into view. In the repro the focused file
   is the first one and the view is at the end, so even the `keepScroll`
   path snaps back after keeping the offset. Fixing only the flag in
   `refreshEntries` would not fix the bug.
3. `strip.setFiles` releases every cell and clears the per-index sets, so
   every visible thumbnail is requested again over IPC: the flicker.

After this work a list update from a rescan or `refreshEntries` keeps the
view where it was (the same files stay under the viewport), never scrolls to
the focused file, and a cell whose path is still listed keeps its loaded
thumbnail even when it moved to another index. The first update after
opening a folder and the resume landing keep their current behaviour: the
offset is 0 at that point anyway, and the resume goes through `show()`,
which still scrolls.

Decision on the focused file leaving the visible range (chosen by the user
over keeping the raw pixel offset): a list update never scrolls to the
focused file. If its cell was inside the viewport before the update it stays
at the same viewport position (the offset shifts by its index delta); if it
was off-screen — the user scrolled away, as in the repro — the view anchors
on the first visible cell instead, so the same files stay in view; if the
anchor path is gone, the raw offset is kept, clamped to the new width. Only
user navigation (`show()`), and a focus forced elsewhere (the focused file
deleted, `mustReshow` -> `show()`), scroll. Rationale: "keep the focused file
visible" is exactly the reported snap, and the wheel already lets the user
scroll away from the focused file; an update must not undo that.

Related `todo.md` items:

- "App: `open entries` is called again after a scan's follow-up rescan":
  left open. This fix makes the follow-up refresh invisible (no snap, no
  thumbnail reload) but does not remove the extra `folder_entries` read or
  its timing cost.
- "App: the backend `folder_entries` read is the main cost of the remaining
  `refreshEntries`": left open, untouched (a backend cost). The frontend
  `set_files=true` share of the `refresh entries:` timing line gets cheaper
  (no cell teardown and reload), but the item is about the backend read.
- Neither item is edited in the implementation commit: `todo.md` is being
  curated in another session (tree-live-watch wrap-up). The wrap-up adds the
  two notes above.

## Steps

- [x] Step 1: Keep the scroll anchor and reuse same-path cells across `strip.setFiles`, and stop `refilter` from scrolling to the focused file on a kept-scroll update
  - Done when:
    - A new pure module `crates/app/ui/src/carry.ts` holds the two decisions
      the strip cannot test itself (vitest runs with `environment: "node"`,
      root `vite.config.ts`, and `strip.ts` touches `document` at module
      scope; `refresh.ts` / `resume.ts` / `idle.ts` are the precedent):
      - `carriedOffset(previous, next, offset, cellWidth, viewportWidth, current)`
        (names free): the new `scrollLeft`. Anchor = `current`'s path when
        its cell overlapped `[offset, offset + viewportWidth)` in
        `previous`, else the path of the first cell overlapping that range;
        result = `offset + (indexIn(next) - indexIn(previous)) * cellWidth`
        when the anchor is still in `next`, else `offset`; always clamped to
        `[0, next.length * cellWidth - viewportWidth]` (minimum 0), the
        clamp `setFiles` already does.
      - `carriedIndices(previous, next, indices)`: `Map<oldIndex, newIndex>`
        for the given old indices whose path is still in `next`.
    - `crates/app/ui/src/carry.test.ts` covers, for the offset: files
      inserted before a visible focused cell (view follows it), focused
      off-screen and files inserted before the view (view follows the first
      visible cell, i.e. the repro: 25 files, view at the end, 100 added),
      anchor deleted (raw offset kept), the clamp when the list shrinks, and
      an empty `next`. For the indices: a reorder, a removal (dropped), an
      insertion (shifted), an untouched list (identity).
    - `strip.setFiles(paths, keepScroll)` in `crates/app/ui/src/strip.ts`:
      - with `keepScroll`, sets `strip.scrollLeft` from `carriedOffset`
        instead of the raw clamped `offset`; without it, still 0.
      - before clearing, collects the cells whose `url !== null` (a loaded
        thumbnail) keyed by their old index, maps them through
        `carriedIndices`, and re-inserts them into `cells` at the new index
        with `el.style.left` updated, marking the new index `requested` so
        `pump` does not ask again. The old, unmapped cells are released as
        today. `render()` then releases any carried cell outside the new
        visible range, as it does for any cell. Cells still in flight or
        without an image are not carried: they were placeholders, so
        re-requesting them (today's behaviour, with the `generation` bump
        dropping the stale response) shows nothing different.
      - the cell's listeners currently close over `index` (`select`,
        `contextMenu`, the name click / `armSlowClick`, `startRename`'s
        `files[index]`), so a moved cell must not keep the old one: `Cell`
        gains an `index` field the listeners read, set by `createCell` and
        updated on the move. Decide during implementation whether to build
        the `cell` object before attaching the listeners or to attach them
        with a closure over a mutable holder; whichever keeps `createCell`'s
        shape closest.
      - the `generation` bump stays on every `setFiles`; carried cells have
        no request in flight, so nothing they own is dropped.
      - the inline-rename resume block after `render()` still works: a
        carried cell at `resume.index` is found by `cells.get`, else one is
        created as today.
    - `strip.setCurrent(index, scroll = true)`: the scroll-into-view runs
      only when `scroll` is true; `render()` and the highlight always run.
      `show()` and `applyPanels` keep the default.
    - `refilter` in `crates/app/ui/src/main.ts`: the non-`show()` branch
      calls `strip.setCurrent(index, !keepScroll)`, and `refreshEntries`
      passes `keepScroll = true` in both its `refilter` calls
      (`refilter(anchor, true, hadPendingResume)` and the `.catch` one). The
      resume landing is unaffected: `hadPendingResume` forces `mustReshow`,
      `show()` -> `setCurrent(index)` scrolls as before. The open-time
      `openDirectory` `strip.setFiles(files)` (offset 0) is unchanged.
    - `docs/agents/tauri-app.md`, entry "Style the strip placeholder on
      `.cell img:not([src])`": the sentence "cells are recreated rather than
      reused on refresh, so `src` is never stale" is reworded to say a cell
      is reused only for the same path, so `src` is never stale. One
      sentence, nothing else in that file.
    - Manually verified on Windows with the repro (25 ARWs, scroll to the
      end, copy 100 in): the view stays on the same files through every
      round, the focused file is not scrolled to, and the visible
      thumbnails do not blank. Also verify: delete the focused file
      externally (focus moves to the neighbour and scrolls to it, as
      before), open a folder with a remembered last-viewed file (the resume
      still lands on it), and `View > Strip` toggled back on (still
      re-centres on the focused cell). This check is the user's, after the
      PR is up.
    - `mise run ci` passes.
  - Implementation approach:
    - The branch is cut from the latest `origin/main`; #521 and #522's
      `main.ts` hunks are in `trashRejected`, `renameFile` and
      `folders.init`, away from `refilter`, `refreshEntries`, `show`,
      `resync` and the scan listeners.
    - Nearby sessions: tree-live-watch wrap-up edits `docs/agents/tauri-app.md`
      (appends entries) and `todo.md`; wait-for-scan step 2 edits
      `settings.ts` / `index.html`; trash-rejected step 2 edits `trash.ts` /
      `tree.ts`. Only the one-sentence `tauri-app.md` edit can touch the same
      file; different region.
    - Keep the diff surgical: no change to how judgments, sharpness, bursts
      or candidates are cleared and re-applied after `setFiles` (`refilter`
      re-applies them per index right after), and no carry of `missing` /
      `ready` / `failed` (today's behaviour, out of scope; see trade-offs).
    - `carry.ts` follows the style of `refresh.ts`: exported pure functions
      with a short comment each, no DOM, no Tauri.
    - `learnings.md` in this folder records anything that surprises during
      the implementation.

## Trade-offs and risks

- **Focused file pushed out of view**: decided by the user (anchored). The
  rejected alternative kept the raw pixel offset only, as today's
  `keepScroll` does; simpler, but when files are inserted before the
  viewport the same offset then shows different files.
- **Reuse the DOM node vs. transfer the blob URL to a fresh cell**: the plan
  moves the cell's element (deterministic, the `<img>` never changes `src`).
  The alternative creates a fresh cell and copies `url` / `img.src` /
  `img.className` from the old one; it avoids the `Cell.index` change but
  relies on the browser's image cache to paint the new `<img>` without a
  placeholder frame, which is likely in WebView2 but not guaranteed. Fall
  back to it only if the listener rework turns out larger than expected.
- **Not carrying `missing` / `ready` / `failed` / in-flight requests**: a
  cell with no image yet is re-requested after each round, as today. This
  costs a few IPC calls per round, not a visible change. Carrying them
  would need the response path to resolve by path instead of the captured
  index (the `generation` bump drops in-flight responses), a larger change
  than this bug needs.
- **Single step**: the scroll fix and the cell reuse are independent but
  both are needed before the repro looks right, and the diff is small.
- **`todo.md`** is not edited in the implementation commit to avoid a
  conflict with the concurrent curation; the wrap-up adds the two notes.

## Progress

- (2026-09-28) Step 1 complete
