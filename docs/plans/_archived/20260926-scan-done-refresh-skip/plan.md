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

# Skip the `scan-done` refresh when the scan changed nothing

## Purpose

Opening an already-indexed, unchanged folder runs `refreshEntries`
(`crates/app/ui/src/main.ts`) twice: once from `openDirectory` and once from
the `scan-done` listener, even though the scan wrote nothing. Measured by the
user (Windows, debug build, D: drive, 2134 indexed RAWs, timing logs on): the
open refresh took 396.8 ms (invoke 385.2 ms, backend `open entries` 321 ms)
and the `scan-done` one 142.1 ms (invoke 136.1 ms, backend 81 ms) with
`scan extract: files=0 done=0`. The `faces-done` refresh is already skipped
in that case (PR #457, `refreshOnFacesDone` in `crates/app/ui/src/refresh.ts`).

After this work the `scan-done` refresh is skipped the same way, but only when
nothing in the index could have changed: the scan pass wrote no row
(`total === 0`) **and** `scan_folder`'s reconcile phase changed no row either.
The second condition is what keeps a deletion-only resync, an externally edited
or deleted sidecar, and the `sidecar-format` reopen correct: those all end with
`total === 0` yet write or delete rows before the scan, and the open-time
`folder_entries` read races them (it goes through the `AppIndexReader`
connection with no ordering against `scan_folder`).

## Steps

- [x] Step 1: Report the reconcile's row changes from `scan_folder` and skip the `scan-done` refresh when neither it nor the scan wrote anything
  - Done when:
    - `ScanStarted` (`crates/app/src/commands.rs`) carries a new field, e.g.
      `changed: usize`, counting the rows `scan_folder`'s prepare phase
      changed: the `files` rows `Index::reconcile` deleted, plus the
      `ratings` rows `reconcile_sidecars_of` wrote (sidecars parsed and stored
      through `store_sidecar_ratings`) or cleared (a gone sidecar over a clean
      row). Zero on the "index unavailable" early return. The existing
      `scan_started_serializes_the_fields_the_frontend_reads` test covers the
      new field, and the `scan reconcile:` / `scan sidecars:` log lines report
      the counts (e.g. `removed=N`, `changed=N`), so the user's log shows why
      a refresh happened.
    - `Index::reconcile` and `Index::reconcile_sidecars` expose those counts;
      their existing tests in `index.rs` and the `reconcile_listed*` helpers
      in `commands.rs` tests keep passing with the mechanical adjustment, and
      one new unit test per count exists (a deletion-only reconcile reports
      the removed count; a cleared sidecar reports one cleared row; an
      unchanged folder reports zero for both).
    - `crates/app/ui/src/refresh.ts` gains a pure decision, mirroring
      `refreshOnFacesDone`: e.g. `refreshOnScanDone(started: ScanStarted | null, scanId: number, total: number): boolean`
      returning `false` only when `started.scanId === scanId`,
      `started.changed === 0` and `total === 0`. Its doc comment states the
      assumptions the way `refreshOnFacesDone`'s does: a `scan-done` total of
      zero means the scan pass wrote no row; `changed === 0` means the
      reconcile deleted no `files` row and wrote or cleared no `ratings` row;
      and the open-time `refreshEntries` read is not ordered against
      `scan_folder`'s reconcile, which is why the skip needs both.
    - `refresh.test.ts` covers: both zero -> skip; scan wrote rows ->
      refresh; reconcile changed rows with total zero (the deletion-only /
      external-sidecar case) -> refresh; the started record belongs to
      another scan -> refresh; no started record -> refresh.
    - `main.ts`: `startScan`'s `.then` records `{ scanId: scan_id, changed }`
      (a `let` next to `scanDone`, reset in `openDirectory` with `scanDone`
      and `progressRefreshedFor`) before it invokes `start_scan`, so it is in
      place before any `scan-done` for that id can arrive. The `scan-done`
      listener keeps everything else it does (`scanErrors`, `scanDone`,
      `scanning`, `renderMeta()`, `strip.refresh()`) and calls
      `refreshEntries()` only when the decision says so.
    - `refreshOnFacesDone` and its tests are unchanged: it still skips on
      the same `scan_id` with both totals zero. When the reconcile changed
      rows, the `scan-done` refresh already re-read them, so `faces-done`
      skipping on the totals stays correct.
    - `learnings.md` records the user's measurement that closes the todo item
      "App: conditionally skip `rebuildExifMenu`'s DOM rebuild in
      `refreshEntries`": `exif=1.1ms` at 2134 rows and 0.1-0.5 ms at smaller
      folders (Windows, debug build, timing logs on), so the DOM rebuild is
      not a cost worth a guard. Put it in the body of the learnings (not under
      `## Deferred issues`) so `todo-curator` can delete the item with that
      reason in the wrap-up; name the todo heading verbatim.
    - `mise run ci` passes (frontend fmt/lint/type-check/vitest and the Rust
      tests). Manual check for the user, written out in the PR: with
      `Timing logs` on, reopen the already-indexed `2026-09-19` folder and see
      one `refresh entries:` line (the open one) followed by `scan-done` with
      no second line; then delete one RAW from the folder (or edit one XMP
      externally) and confirm the watcher's resync produces a
      `refresh entries:` line and the strip / rating reflects the change.
  - Implementation approach (as far as it is known):
    - Backend counts. `Index::reconcile` (`index.rs:584`) has the deleted
      rows in its loop; `Index::reconcile_sidecars` (`index.rs:1106`) has the
      cleared rows in its `None` arm; `reconcile_sidecars_of`
      (`commands.rs:303`) has `parsed.len()` (the rows `store_sidecar_ratings`
      is given; count them as changed even though a row that went dirty in
      the parse window is left alone, which is the conservative side). Use
      counts (not a `bool`), returned alongside the existing values (a small
      struct or tuple); adjust the test call sites mechanically. Do not add a
      query-based before/after `COUNT(*)`; the counts are already in hand
      inside the transactions.
    - `ScanStarted` is a plain `serde::Serialize` struct; add the field and
      extend the serialization test. The three `Ok(ScanStarted { .. })`
      returns in `scan_folder` all need it (0 on the no-index return).
    - Frontend. Follow `refreshOnFacesDone` exactly in shape and naming:
      export an interface for the recorded value (e.g. `ScanStarted { scanId; changed }`
      next to `ScanDone`), keep the decision pure, and keep `main.ts` to
      wiring. The `.then` in `startScan` is guarded by `seq !== currentScan`;
      record the value after that guard. `openDirectory` resets it with the
      other per-folder scan state (`scanDone = null; progressRefreshedFor = null;`).
    - Do not change the `scan-progress` gating (`refreshOnProgress`) or the
      `faces-done` listener.
    - Consistency with the guide (`docs/agents/tauri-app.md`, "Two folder
      paths" and "The folder watcher cannot loop on the app's own sidecar
      writes"): an app-side rating write marks the row written with the
      sidecar's stat, so a later resync parses nothing and `changed` stays 0;
      the skip therefore does not fire a refresh on the app's own writes.
    - Out of scope (mention in the PR as a possible follow-up only): the
      backend `folder_entries` read itself (`open entries` 321 ms cold on
      2134 rows), which remains the cost of the one refresh that stays.

## Trade-offs and risks

- Frontend-only skip on `total === 0` (not taken): simplest, but it would
  hide an externally edited or deleted sidecar picked up by a resync, a
  deletion-only resync's stale rows, and the `sidecar-format` reopen's
  re-parsed ratings, because the open-time read is not ordered against the
  reconcile. The backend count is the price of a correct skip.
- Counting `parsed.len()` rather than the rows `store_sidecar_ratings`
  actually updated over-reports in the rare parse-window race (a `set_rating`
  landing while the lock is released), which only costs one extra refresh.
- If a future backend change writes index rows in the prepare phase without
  going through the two reconciles (or in the scan pass without raising
  `total`), the skip would hide them; the doc comment on the decision states
  the assumption, as `refreshOnFacesDone`'s does for `write_faces`.
- Signal alternative not taken: comparing `allFiles` before and after
  `list_arw` in the frontend. It detects added/removed RAWs but not sidecar
  changes, so it cannot replace the backend count.

## Progress

- Step 1: Shipped `ScanStarted.changed` (the reconcile's dropped `files` rows
  plus the sidecar reconcile's written/cleared `ratings` rows, plus 1 when
  `scan_folder` joined a previous scan that was still running, whatever
  folder it was scanning), the `refreshOnScanDone` decision in `refresh.ts`,
  and wired `main.ts` to skip the `scan-done` refresh only when both
  `changed` and `total` are zero. Fixed a `clippy::type_complexity` CI
  failure by adding a `SidecarToParse` type alias (recorded in
  `learnings.md`).
- (2026-09-26) Step 1 complete
