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

# Strip sort order (capture time, file name, rating)

## Purpose

`list_arw_in` (`crates/app/src/commands.rs`) and `Index::entries`
(`crates/app/src/index.rs`) return files in file-name order, and the strip,
paging and the filter all work on that order. A counter rollover
(`DSC09999` -> `DSC00001`) or two bodies sharing a folder breaks the
shooting sequence, which is what a culling pass wants to walk. This work adds
a sort choice — capture time, file name, rating — to the strip pane, with
paging, navigation and the filter following the chosen order. It closes the
`todo.md` item "App: the strip is always in file-name order".

Current state the plan is based on:

- Ordering is already a frontend concern. `crates/app/ui/src/main.ts` keeps
  `allFiles` (the `list_arw` result) and derives `files = allFiles.filter(passes)`
  in `refilter()` and `openDirectory()`; `index`, `move()`, `strip.setFiles`,
  `fileIndex` and the anchor fallback in `refilter()` all work on `files`.
  "Paging" is `move(delta)` over `files`, so it follows whatever order `files`
  has.
- Everything needed for the sort keys is already in the frontend:
  `entries: Map<path, IndexedFile>` carries `capture_time` (EXIF
  `DateTimeOriginal`, `YYYY:MM:DD HH:MM:SS`, lexicographically ordered) and
  `subsec` (EXIF `SubSecTimeOriginal`, a digit string of fractional seconds
  with no fixed length); `ratings: Map<path, number>` carries `-1` (reject) or
  `1`-`5`, absent meaning unrated; `picks` the pick flag. `entries` fills in
  as the scan progresses (`refreshEntries()` after `scan-progress` /
  `scan-done`, followed by `refilter()`), and is complete immediately on a
  folder that has been scanned before.
- The index has a `files_capture (capture_time, subsec)` index, but nothing
  reads it; no schema change is needed.
- The filter menu (`#filter` / `#filter-menu` in `crates/app/ui/index.html`,
  styled in `crates/app/ui/style.css`) is the existing in-pane dropdown; its
  items are `role="menuitemcheckbox"` buttons with `aria-checked`, and
  `filterChanged()` mirrors state onto them and calls `refilter()`.
- The frontend has Vitest tests (`crates/app/ui/src/exif.test.ts`, run by
  `vp test`) for pure modules; `main.ts` itself is untested.
- Settings persisted in the store (`lastFolder`, `sidecarFormat`,
  `shortcuts`) each have a small command pair in `commands.rs`
  (`last_folder` / `remember_folder` is the simplest template).

Decisions confirmed with the user: default order is file name; the choice is
persisted across launches (Step 3); the control lives in the strip pane, not
the native menu bar.

## Steps

- [x] Step 1: A pure ordering module with tests
  - Done when:
    - `crates/app/ui/src/sort.ts` exports `type SortKey = "name" | "capture" | "rating"`
      and one function that orders paths, e.g.
      `orderFiles(key, paths, lookup)` where `lookup(path)` returns the
      `{ captureTime, subsec, rating }` the caller knows for that path (all
      optional). It never mutates its input and is total: every path gets a
      position whatever is missing.
    - Orders, each with the file name (`baseName`, as the existing
      `main.ts` helper computes it) as the final deterministic tie-break:
      - `name`: file name, byte-wise like today's `list_arw` (`Path::file_name`
        `Ord`), so the order is unchanged from the current behaviour.
      - `capture`: `capture_time` ascending, then `subsec` as a fraction
        (compare `"0." + subsec` numerically, or pad the shorter string with
        trailing zeros; `"5"` is `.5`, `"122"` is `.122`), then file name.
        Files without a capture time (not yet scanned, or an error row) go
        after every timed file, in file-name order.
      - `rating`: 5 stars first down to 1, then unrated, then rejected
        (`-1`); the pick flag does not affect the order; then file name.
    - `crates/app/ui/src/sort.test.ts` covers: a rollover folder
      (`DSC09998`, `DSC09999`, `DSC00001` with ascending times) comes out in
      time order under `capture`; two bodies interleave by time; equal
      `capture_time` is split by `subsec` and `"5"` sorts after `"122"`;
      missing capture times trail in name order; `rating` places stars
      descending, unrated before rejected, ties by name; `name` equals the
      input order of an already name-sorted list; the input array is not
      mutated.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep it free of DOM and of `main.ts` imports so Vitest runs it as is
      (the pattern of `exif.ts` / `exif.test.ts`). Import with `.js`
      suffix per `docs/agents/tauri-app.md`.
    - Sort a copy with a comparator; JavaScript's sort is stable but the
      comparator must still end in the file-name compare so the result does
      not depend on input order.

- [x] Step 2: Sort choice in the strip pane, applied to the strip, paging and the filter
  - Done when:
    - `main.ts` keeps a `sortKey: SortKey` (default `"name"`) and an ordered
      view of the folder: `refilter()` and `openDirectory()` build `files` as
      `orderFiles(sortKey, allFiles, ...)` filtered by `passes`. The anchor
      fallback in `refilter()` ("the next passing file after it, or the last
      one before it") searches the ordered list, not `allFiles`, so it
      follows the chosen order.
    - A sort control sits in the strip pane beside the filter toggle: a
      second dropdown (`#sort` / `#sort-toggle` / `#sort-menu`) with three
      `role="menuitemradio"` items (Capture time, File name, Rating), reusing
      the filter menu's classes/CSS (open/close, click-outside close,
      `Escape` close, `blur()` after click so `Space` does not re-press it,
      `aria-checked` for the selected item). Choosing an item sets
      `sortKey` and calls `refilter()`; the current file stays current
      (the anchor already does that).
    - The order updates as capture times land: `refreshEntries()` already
      calls `refilter()`, so a folder being scanned for the first time under
      `capture` re-sorts as rows arrive. Verify during implementation that
      the resulting `strip.setFiles` calls (which drop every thumbnail cell
      and re-request the visible ones) are tolerable at the 10/s
      `scan-progress` rate on a first scan of a large folder, and record
      the observation in `learnings.md`. If it is not, only re-sort on
      `scan-done` (keep the `refilter` on progress for the filter but skip
      the re-order until the scan finishes) and say so in the README.
    - `sortKey` is kept across folder opens within the session (like
      `shownFlags` / `shownStars`), and the ordered list is rebuilt on open.
    - `IndexedFile` in `main.ts` already declares `capture_time` / `subsec`;
      no backend change.
    - **(manual, GUI automation is unavailable)** the user confirms on a
      folder containing a counter rollover or two bodies: `Capture time`
      walks the shooting sequence; `Rating` groups 5 stars first and
      rejects last; paging with the arrow keys, clicking a strip cell and
      the `n / N` position all follow the chosen order; a filter narrows
      the ordered list without changing its order; judging a file under
      `Rating` moves it and the cursor stays on it.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Per the key/menu policy, no keyboard shortcut: the sort is a view
      setting, not a culling action, and lives in the pane menu like the
      filter. It is not put in the native menu bar because `app_menu` is
      built without state and a radio group there would need a backend
      event round trip and state sync for a purely frontend setting.
    - Do not touch `crates/app/src/commands.rs` `list_arw_in` or
      `Index::entries`; their file-name order stays the base order.
    - `refilter()`'s early return (`next` identical to `files`) already
      covers "nothing changed"; a pure re-order produces a different array
      and goes through `strip.setFiles` like a filter change does.

- [x] Step 3: Persist the chosen sort in the settings store
  - Done when:
    - A `sortOrder` key in the settings store (`"name"` / `"capture"` /
      `"rating"`) is read at launch and written on every change, through a
      `sort_order` / `set_sort_order` command pair in
      `crates/app/src/commands.rs` mirroring `last_folder` /
      `remember_folder` (registered in `main.rs`'s handler list and, if the
      capability file lists commands, there). An unknown or missing value
      falls back to the default.
    - `main.ts` applies the stored value before the first folder open
      (`reopenLastFolder()` already runs at startup; read the sort first or
      re-run `refilter()` once it resolves) so the reopened folder comes up
      in the remembered order.
    - A Rust unit test in `commands.rs` covers the fallback for an unknown
      stored value (only if the parsing lives in a testable function; if it
      is a one-line `match`, no test).
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 2 is merged.
    - Keep the value a plain string, not a settings-window control; the
      pane menu is the only UI.

- [ ] Step 4: Documentation and todo close-out
  - Done when:
    - `README.md`'s feature list gains a **Sort menu** entry beside the
      filter menu: the three orders, the tie-break rules in one line
      (capture time then sub-second then file name; rating descending with
      rejects last), that files without a capture time trail, that the
      order re-settles while a folder is scanned for the first time (or
      only on scan end, per Step 2's finding), and that the choice is
      remembered across launches. No phase or verification notes (README
      content policy).
    - `todo.md`: the "App: the strip is always in file-name order" section
      is removed (or its TODO checked and the section trimmed, whichever
      matches how neighbouring closed items were handled in the last
      curate commit).
    - `CLAUDE.md` "Layout" is untouched unless a Rust file was added;
      `docs/agents/tauri-app.md` gains any pitfall Steps 1-3 hit (or
      nothing).
    - `mise run ci` passes.

## Trade-offs and risks

### Where the sort is computed: frontend (chosen) versus a backend `ORDER BY`

- **Chosen: frontend.** `files` is already derived in the frontend from
  `allFiles` plus `entries` / `ratings`, the filter is frontend-only, and a
  rating change is applied to the frontend maps before the backend hears
  of it. Sorting in the same place keeps one source of order and needs no
  IPC or schema change; sorting 5000 strings is sub-millisecond.
- **Alternative: `ORDER BY` in `Index::entries` and a sorted `list_arw`.**
  Would need a sort parameter on two commands, and the order of a file
  judged a moment ago would lag the backend write. Not worth it for this
  data size.

### Default order: file name (chosen by the user)

- File name keeps today's behaviour and never re-sorts while a folder is
  scanned for the first time. The persisted choice (Step 3) lets a user who
  wants capture order keep it.

### Re-sorting during a first scan

Under `capture` (and `rating` on a first open, before `folder_entries`
delivers sidecar ratings), rows arrive over the scan and the order settles
progressively. Each re-order goes through `strip.setFiles`, which discards
every thumbnail cell and re-requests the visible ones; on a 5000-file first
scan that could visibly flicker. Step 2 measures this and falls back to
"re-order on `scan-done` only" if needed. A folder already scanned is
unaffected: `entries` arrives in one refresh.

### Rating order details

- Pick flag: ignored in `rating` order (a pick coexists with stars).
- Judging under `Rating` moves the judged file to its new group; the cursor
  follows it (the anchor rule). Auto-advance is a separate `todo.md` item and
  out of scope here.

### Tie-break for capture time when `subsec` is missing on one side

A file with a time but no `subsec` is treated as `.0`, sorting before a
sibling at the same second with a `subsec`. Fine for one body; mixed bodies
where one writes no `subsec` fall back to file name within the same second.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-19) Step 2 complete
- (2026-09-20) Step 3 complete
