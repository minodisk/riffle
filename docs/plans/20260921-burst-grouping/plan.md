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

# Burst grouping: time-based groups on the strip, burst navigation and reject-rest

## Purpose

A continuous-shooting run lands on the strip as a flat list of near-identical
frames, and culling it means paging through them one by one, picking one and
rejecting the others by hand. After this work Riffle groups consecutive frames
whose capture timestamps lie within a gap threshold into a burst, draws the
burst on the strip so its extent is visible, lets the user jump from burst to
burst, and rejects every other frame of the current burst with one key once a
frame has been chosen.

This is phase 1 of a larger roadmap (similarity-based split / merge, then
face / person scoring). Only time-based grouping is built here. The grouping
is a pure frontend function over per-file facts that already come from the
folder index, so a later phase can add further per-file features to the index
(a perceptual hash, a face score) and refine the same grouping without moving
it.

## Facts established while planning

- `SubSecTimeOriginal` is already parsed (`crates/core/src/arw.rs`,
  `Shot::subsec`) and stored (`files.subsec` in `crates/app/src/index.rs`,
  with the `files_capture (capture_time, subsec)` index). Nothing in core or
  the index has to change for the timestamp.
- Both `capture_time` (`YYYY:MM:DD HH:MM:SS`) and `subsec` reach the frontend
  as `IndexedFile.capture_time` / `subsec` through `folder_entries`;
  `sort.ts` (`orderFiles("capture", ...)`) already orders by capture time,
  then zero-padded subsec, then file name, with files lacking a capture time
  last.
- Verified with `riffle-cli info`: the Sony α7 V ARW carries a subsec
  (`122`); the Leica M11-P DNG carries none. So Leica frames have 1 s
  resolution only, and a missing subsec has to count as `0 ms`, with the
  gap test inclusive (`<= threshold`), so two Leica frames stamped at seconds
  `N` and `N+1` stay in one burst.
- `capture_time` is wall-clock with no zone. Gaps are differences between two
  such stamps of the same folder, so parsing with `Date.UTC` is enough; no
  time zone handling.
- The strip (`crates/app/ui/src/strip.ts`) is a virtual list over a fixed
  `--cell-height` (`style.css`). Its per-index stores (`ratings`, `picks`,
  `labels`, `sharpness`) are cleared by `setFiles` and re-fed by `main.ts`
  after every `refilter`. A burst store follows the same pattern.
- `main.ts` keeps `entries: Map<string, IndexedFile>` and refreshes it in
  `refreshEntries` (called on open, resync, scan progress / done and
  `index-cleared`); a derived state must be recomputed there and after
  `refilter` (see `docs/agents/tauri-app.md`, "A derived-state refresh has to
  run even when `refilter` short-circuits").
- Undo / redo (`History<Judgement>` in `main.ts`, `undo.ts`) hold one file
  per entry; `step()` pops one and re-applies it through `commit`.
- Keymap: actions live in `DEFAULTS` in `crates/app/src/shortcuts.rs` (the
  panel order), with a `defaults` test that enumerates every action; the
  settings window names them in `shortcutLabels` in
  `crates/app/ui/src/settings.ts`; the README lists them in "Keys". Plain
  `arrowleft` / `arrowright` are unbound today; `ctrl+arrow*` are reserved on
  macOS; `shift+x` is free everywhere.
- Policy (auto memory): culling actions are rebindable keys, non-culling
  actions go in the menu. Burst navigation and reject-rest are keys; no menu
  item is added.

## Decisions

- Grouping is computed in the frontend, not stored in SQLite: the inputs
  (`capture_time`, `subsec`) are already cached in the index and shipped on
  every `folder_entries`, and a group boundary depends on neighbours, so an
  index column would have to be rewritten for the whole folder on every scan
  diff. See "Trade-offs and risks" for the alternative.
- The threshold is a constant (`BURST_GAP_MS = 1000`) exported from
  `burst.ts`, not a setting. Leica files only carry whole seconds, so any
  value below 1 s is meaningless for them, and the sharpness cue's window
  radius is a constant for the same reason. Revisit once similarity grouping
  exists; see "Trade-offs and risks".
- Bursts are drawn in place on the strip (a bracket on the cells), never
  collapsed: the virtual list depends on a fixed cell height.
- Grouping runs over `allFiles` in capture order, independent of the chosen
  sort and filter; the strip shows the burst of each *displayed* cell, so a
  filtered or rating-sorted strip still marks members correctly.

## Steps

- [x] Step 1: Group files into bursts and show the groups on the strip
  - Done when:
    - `crates/app/ui/src/burst.ts` (new, no DOM, no Tauri) exports
      `BURST_GAP_MS` and a pure `groupBursts(paths, lookup, gapMs)` that
      returns the bursts as arrays of paths in capture order (or a
      `Map<path, {burst, position, size}>`; pick the shape `main.ts` finds
      most useful and record why). Rules: order by capture time, subsec,
      then file name (reuse `orderFiles("capture", ...)` from `sort.ts`
      rather than re-implementing the tie-break); a missing subsec is
      `0 ms`; a new burst starts when the gap to the previous file is
      `> gapMs`; a file with no capture time is never in a burst with
      another (a singleton at the end, matching the sort order)
    - `crates/app/ui/src/burst.test.ts` covers at least: a gap exactly at
      the threshold stays in the burst and one millisecond over splits it;
      a run across a second / minute / day boundary; subsec ordering
      (`"5"` sorts before `"122"` within the same second, so a burst whose
      files are named out of capture order is still grouped by time); a
      missing subsec on some or all files (the Leica case: consecutive
      whole seconds stay grouped); files without a capture time are
      singletons and do not break a burst around them; a lone file is a
      burst of one
    - `main.ts` computes the grouping whenever `entries` is refreshed
      (`refreshEntries`) and hands the strip, after every `refilter` and
      inside `refilter` after `setFiles`, per displayed index whether the
      cell belongs to a burst of size > 1 and whether the previous / next
      displayed cell is in the same burst (so the bracket opens and closes
      correctly when a filter or the rating sort separates members)
    - `strip.ts` gets a `setBurst(index, value | null)` mirroring
      `setSharpness`, with its own per-index store cleared in `setFiles`,
      and paints it on the cell (a `burst` class plus `burst-first` /
      `burst-last`, or equivalent)
    - `style.css` draws a burst as a thin bracket down the cell's right edge
      (the left edge is the sharpness bar) that is continuous across
      adjacent member cells, i.e. it spans the `--cell-gap` between them and
      is capped on the first and last cell. Only bursts of two or more
      frames are drawn
    - The `#position` line reads `N / M` as today, followed by the position
      in the burst (`· 3 / 7 in burst`, or similar) when the current file
      is in a burst of two or more
    - `mise run ci` passes
  - Implementation approach:
    - Parse `YYYY:MM:DD HH:MM:SS` with `Date.UTC`; treat a stamp that does
      not parse like a missing one
    - `subsec` to milliseconds the way `compareSubsec` in `sort.ts` pads:
      `"1"` is 100 ms, `"12"` is 120 ms, `"122"` is 122 ms; more than three
      digits truncates
    - Keep the per-index value the strip receives small and derived, as
      `RelativeSharpness` is; the strip must not learn about paths or
      timestamps
    - Check with a real burst folder (the Sony ARWs and the Leica DNGs under
      `~/Downloads`) that the bracket appears where expected; note the
      observed group sizes in `learnings.md`

- [x] Step 2: Burst navigation keys
  - Done when:
    - Two new keymap actions, `burstPrevious` and `burstNext`, are added to
      `DEFAULTS` in `crates/app/src/shortcuts.rs` right after `next` (the
      panel order), with defaults `arrowleft` / `arrowright`; the `defaults`
      test and any other test enumerating actions are updated
    - `shortcutLabels` in `crates/app/ui/src/settings.ts` names them
      (`Previous burst`, `Next burst`)
    - `main.ts` dispatches them: `burstNext` moves to the first displayed
      file of the next burst after the current file's burst (a singleton is
      its own burst, so the key also walks single frames); `burstPrevious`
      moves to the first displayed file of the current burst when the
      current file is not its first, else to the first displayed file of
      the previous burst (the "go to the start, then to the previous track"
      behaviour). Both clamp at the ends and work on the displayed, filtered
      `files` list, so hidden members are skipped
    - The step function that finds the target index is a pure exported
      function in `burst.ts` (given the displayed list's burst ids and the
      current index) and is unit-tested in `burst.test.ts`: from inside a
      burst, from a singleton, at either end, and with members hidden by a
      filter (non-adjacent ids)
    - The README "Keys" table gains both rows
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Step 1 is merged
    - Paging goes through the existing `move`/`show` path so the zoom state
      and page timing behave as for `next` / `previous`; set `index` and
      call `show()` the way `move` does
    - Check the key names against `forbidden()` and the macOS reserved list
      (`ctrl+arrowleft` is reserved; plain `arrowleft` is not)

- [x] Step 3: Reject the rest of the burst, undone as one entry
  - Done when:
    - A new keymap action `rejectRest` is added to `DEFAULTS` right after
      `reject`, default `shift+x`; `shortcutLabels` names it (`Reject the
      rest of the burst`); the `defaults` test is updated
    - Pressing it on a file in a burst rejects every other member of that
      burst that is not already rejected (`rating -1`, `pick false`, label
      kept, exactly as the `reject` key does per file) and leaves the current
      file untouched. On a singleton, or when nothing changes, it does
      nothing (no history entry, no invoke)
    - The judgements are applied locally first, then the strip is refiltered
      **once**, then one `set_rating` invoke per changed file goes to the
      backend (the existing command; the sidecar writer coalesces them), each
      reverting its own file on failure as `commit` does today
    - One `Edit > Undo` restores all of them: the history entry type becomes
      a batch (`Judgement[]`; a single-file judgement pushes a one-element
      batch), `step()` re-applies every file of the batch and pushes the
      batch's current states onto the other stack, and the status line says
      how many files were undone / redone when more than one. `undo.ts`'s
      `History` needs no change beyond the type parameter; `remove` on
      failure removes the batch that contains the failed file only when
      every file of it failed, otherwise it keeps the batch (record the
      choice made and why in `learnings.md`)
    - `undo.test.ts` (or a new pure module test) covers a batch push / pop
      and a batch removal
    - Auto-advance does not fire for `rejectRest` (`advancesAfter` stays as
      it is); the current file stays current unless the filter now hides it
    - The README "Keys" table gains the row, and the "Undo" bullet says a
      reject-rest is undone as one
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Step 1 is merged (Step 2 is independent of this step)
    - Members are taken from the grouping over `allFiles`, so a member
      hidden by the active filter is rejected too (see "Trade-offs and
      risks" for the alternative); a member already rejected is skipped so
      the batch only holds real changes
    - Split `commit` into "apply locally" and "invoke" parts (or add a
      `commitMany`) rather than calling `commit` per file: `commit` calls
      `refilter` every time, and with a `Rejected` filter each call would
      rebuild the strip
    - Follow "Carry every judgement field on every write" and the
      `labelKnown` rule in `commit` for each file
    - Follow "Undo must re-anchor conditionally": anchor on the current file
      only if it still passes the filter

- [ ] Step 4: Documentation
  - Done when:
    - `README.md` "Features" gains a `Bursts` bullet: what a burst is (frames
      whose capture times are within 1 s of the previous frame, in capture
      order, whatever the chosen sort), that Leica files are grouped at
      whole-second resolution because they carry no sub-second time, how it
      is drawn, the burst position on the counter, the two navigation keys
      and reject-rest (with its undo). Written to the README content policy:
      user-facing only
    - `CLAUDE.md` "Layout" mentions `burst.ts` beside the other named
      frontend modules if the layout paragraph lists any; otherwise leave it
    - `mise run ci` passes (lychee checks the Markdown)

## Trade-offs and risks

- **Where the grouping lives (decided: frontend).** Storing a burst id in the
  SQLite index would cache it, but the inputs are already cached and shipped,
  and a boundary depends on neighbours, so the column would be recomputed for
  the whole folder on every scan diff and need a schema bump plus migration
  test. The later similarity phase can cache its *per-file* features (a
  hash) in the index and still combine them in the frontend grouping. If the
  caller prefers the index route, Step 1 grows a `files.burst` column, a
  `SCHEMA_VERSION` bump with the v7-style `files` drop, and a regroup pass at
  the end of `start_scan`.
- **Threshold as a setting (decided: constant).** A settings-window field
  would need a store key, a command pair and an event like `auto-advance`,
  for a value the Leica case cannot use below 1 s. If the caller wants it
  configurable anyway, add it as a `burstGapMs` key in the settings store in
  a separate step after Step 1, mirroring `set_auto_advance`.
- **Camera body as a burst boundary (open).** Two bodies shooting at once in
  one folder would interleave into one burst. `IndexedFile.exif.camera` is at
  hand, so starting a new burst when the camera label changes is one
  comparison in `groupBursts`. It is not in the steps because no such folder
  exists to verify it and the user's folders are single-body; the caller can
  add it to Step 1's rules (one extra test) if wanted. Sony's maker-note
  sequence number is not parsed today and is not needed for the time-based
  phase.
- **Reject-rest and hidden members (open).** Rejecting members the filter
  hides is what "the rest of the burst" means, but the user does not see it
  happen. The alternative is to reject only the displayed members; the
  difference is one `passes` check in the member selection. The plan takes
  "all members" and undo covers it as one entry; the caller may flip it.
- **Reject-rest replaces picks.** Like the `reject` key, it turns a picked
  sibling into a reject. The alternative, skipping picked members, protects
  a deliberate earlier pick but makes the action's result depend on state
  the user may not see. Not addressed beyond undo.
- **"Comparing" is within-burst paging.** The 1:1 focus check stays on while
  paging and the sharpness cue already ranks neighbours, so comparing frames
  of a burst is `next` / `previous` with `z` on; a side-by-side compare view
  is out of scope for this plan.
- **The sharpness cue's window is unchanged.** It still compares a frame with
  two neighbours on each side regardless of burst boundaries. Aligning it to
  the burst is a natural follow-up but is not in scope.
- **Default keys.** `arrowleft` / `arrowright` are free today but a later
  free-pan feature might want them; `shift+x` is free everywhere. All three
  are rebindable, so the cost of a later change is a default swap.
- **Default keys and the burst position line are unverified on a device** in
  the planner's environment (GUI automation does not work on this Mac); Step
  1 and 3 should be exercised by hand on a real burst folder.

## Progress

- (2026-09-21) Step 1 complete
- (2026-09-21) Step 2 complete
- (2026-09-21) Step 3 complete
