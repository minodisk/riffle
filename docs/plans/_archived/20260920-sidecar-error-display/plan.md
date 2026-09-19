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

# Show sidecar problems in a sticky, dismissible error area

## Purpose

Two `todo.md` items share one root cause: the app has no place to show a
sidecar problem that outlives the next keypress.

- **"App: a `sidecar-error` event can be missed under key-mashing."** The
  `sidecar-error` handler in `crates/app/ui/src/main.ts` calls `setStatus`,
  which writes the single transient `note` slot that the next page turn,
  undo or judgement failure overwrites, so a write failure raised while the
  user is mashing rating/paging keys goes unseen. Its second TODO ("revert
  the optimistic has-sidecar flag") is already closed: #54 (`f1ba56d`)
  removed the `sidecars` set and the sidecar header from the meta pane, and
  `has_sidecar` survives only as an unused field of the `IndexedFile`
  interface. Nothing is left to revert; this plan records that rather than
  adding code for it.
- **"App: an unparseable or oversize sidecar fails silently on folder
  open."** `reconcile_sidecars_of` in `crates/app/src/commands.rs` drops the
  `Err` of a sidecar it cannot read or parse (`.ok()?` inside a `filter_map`)
  and silently skips one over `MAX_SIDECAR_BYTES`, so the file shows no
  rating and no reason.

After this work a sidecar write failure stays on screen until the user
dismisses it (or the same file's error is superseded, or another folder is
opened), and opening a folder with a corrupt or oversize sidecar lists those
sidecars in the same area.

Removing the two resolved `todo.md` headings happens in wrap-up, not in a
step.

## Steps

- [x] Step 1: Add a sticky error area to the meta pane, route `sidecar-error` into it, and report unparseable/oversize sidecars from `scan_folder`
  - Done when:
    - A new pure module `crates/app/ui/src/errors.ts` holds the error list's
      logic (add with a key so a later error for the same key replaces the
      earlier one; dismiss one by key; clear all; list in insertion order,
      a superseded entry keeping its original position or moving to the end,
      whichever the implementation picks, pinned by a test), with
      `crates/app/ui/src/errors.test.ts` (vitest, same pattern as
      `undo.ts`/`undo.test.ts`) covering: add, supersede by the same key
      keeps one entry with the newest message, dismiss removes only that
      key, clear empties, and order.
    - `crates/app/ui/src/main.ts` keeps one instance of it, renders every
      entry in `renderMeta()` (or a sibling element inside `#meta`) as a
      line visually distinct from `.note` (its own class in
      `crates/app/ui/style.css`; a warning colour rather than `#999`) with a
      per-entry dismiss button, and the list survives `setStatus`, paging,
      judgement keys and `scan-progress`/`scan-done` renders. It is cleared
      when `openDirectory` starts a new folder open (the same place
      `touched`/`history` are cleared) and, as today's handler does, an event
      for a path not in `allFiles` is ignored.
    - The `sidecar-error` handler adds to that list, keyed by `payload.path`,
      with the message it shows today (`${baseName(path)}: ${message}`),
      and no longer calls `setStatus`.
    - `reconcile_sidecars_of` returns, alongside the dirty rows, the
      sidecars it could not use: one entry per sidecar that failed
      `std::fs::read`, `read_rating`, `read_pick` or `read_label`, or that
      exceeded `MAX_SIDECAR_BYTES`, each carrying the sidecar path and a
      message (the parser's `Err` string, or a fixed "larger than 4 MiB"-style
      text for oversize). The row-handling behaviour (unparsed rows keep
      their old stat; oversize rows are kept out of the writer's list) is
      unchanged and the existing tests still pass.
    - `ScanStarted` (the `scan_folder` return value) gains a
      `sidecar_errors` field (a `Vec` of a small `serde::Serialize` struct
      `{ path, message }`; empty on the early-return paths), and the
      frontend's `scan_folder` `.then` adds each entry to the same error
      list, keyed by the sidecar path, formatted as
      `${baseName(path)}: ${message}`, after the existing `token !==
      folderToken` guard.
    - Rust tests in `commands.rs`: the existing
      `an_oversize_dop_is_neither_read_nor_handed_to_the_writer` also asserts
      the oversize sidecar is reported by path; a new test writes an
      unparseable sidecar (a truncated XMP document, as
      `crates/core/src/xmp.rs`'s `a_truncated_document_is_an_error` does) next
      to a RAW, asserts it is reported with the sidecar path, that the
      file's stored rating is untouched, and that a second reconcile of the
      unchanged folder reports it again (the row's stat is never updated for
      an unparsed sidecar, so the problem recurs on every open, which is the
      intended behaviour); a serialisation test pins `ScanStarted`'s JSON
      shape (`serde_json` is already a dependency), so the field names the
      frontend reads are covered.
    - `README.md`'s sentence about a sidecar that could not be written
      (around line 167, "Ratings and sidecars") says where the error is shown
      and that it stays until dismissed, and mentions that a sidecar the app
      cannot read on open is listed there; one or two sentences, user-facing
      only.
    - `learnings.md` records that the "revert the has-sidecar flag" TODO was
      already closed by #54, with the commit reference, so wrap-up can drop
      both headings.
    - `mise run ci` passes.
  - Implementation approach:
    - Frontend state: follow the `History` class in `undo.ts` (a tiny class
      with the logic, exported, no DOM). Keep the DOM work in `main.ts`, in
      or next to `renderMeta()`, which already rebuilds `#meta` with
      `replaceChildren()` on every render, so a per-entry `button` created
      inside the render (with its click handler attached there) needs no
      separate lifecycle. Do not add a keyboard shortcut for dismissing: keys
      are reserved for culling (see the key/menu policy in auto memory) and
      a click is enough.
    - Ordering inside the pane: after the metadata rows and the transient
      `note`/`scanning` lines is fine; the 1:1 indicator's comment explains
      why it is driven by `zoomed` rather than `note`, and the same reasoning
      applies here (the error list is its own state, never touched by
      `setStatus`).
    - Keep `setStatus` for everything else (undo notes, drop hints, invoke
      failures). Only sidecar problems move to the sticky area; do not widen
      the scope.
    - Rust: turn the `filter_map` in `reconcile_sidecars_of` into a loop that
      pushes to either `parsed` or a `problems` vector; build the oversize
      entries from the existing `oversize` set (the path there is the RAW
      path; report the sidecar path from `to_parse`'s `SidecarStat`). Return
      a small struct or a tuple; the 17 test call sites go through the
      `reconcile_listed` helper, so change the helper (or add a second one)
      rather than every test.
    - `scan_folder` currently matches `dirty` with `Ok(dirty) => ... queue
      to writer` / `Err(e) => log::error!`; carry the problems out of the `Ok`
      arm into the final `Ok(ScanStarted { .. })` and into the superseded
      early return (`state.latest_id != scan_id`) too, since the frontend
      drops that result by token anyway. The whole-reconcile `Err` path stays
      a log line (it is a database failure for the folder, not a per-sidecar
      problem).
    - `docs/agents/tauri-app.md` needs no change unless a new pitfall is hit.

## Trade-offs and risks

- **Carrier for reconcile problems: `ScanStarted` (chosen) versus a
  `scan-done` count or a dedicated event.** The todo item suggested
  `scan-done`, but the problems are known inside `scan_folder` before
  `start_scan` runs, whereas `scan-done` is emitted from `run_scan`'s
  completion thread and would need the list threaded through `PendingScan`
  and `Done`. Returning them from the command is the smallest change and
  arrives before the scan starts, so the user sees the problem at once
  rather than after a possibly long first scan. Downside: a folder whose
  index cache is unavailable (`AppIndex` is `None`) reconciles nothing and
  reports nothing, which is also true today.
- **Names versus a count.** The plan reports names (the sidecar path, shown
  as its base name) because a count alone gives the user nothing to act on.
  A folder with hundreds of corrupt sidecars would produce hundreds of lines
  in a pane that scrolls (`#meta { overflow-y: auto }`); acceptable for a
  rare failure, and each line can be dismissed.
- **Persistence across a folder reopen.** Clearing the list on
  `openDirectory` means a write error for folder A disappears when the user
  opens B; the judgement itself is still in the index and is retried on the
  next open of A (existing behaviour). Keeping errors across folders would
  need per-folder keys; not worth it now.
- **Where `sidecar-error` is ignored.** Today's handler drops an event whose
  path is not in `allFiles` (a stale write for a previous folder). The plan
  keeps that guard.
- **The already-closed TODO (b).** The plan does not delete the unused
  `has_sidecar` field from the `IndexedFile` TypeScript interface: it mirrors
  the Rust struct field-for-field.
- **Single PR.** Roughly 250-300 lines with tests. If review finds it too
  large, split at the `ScanStarted` boundary: PR 1 = error area +
  `sidecar-error` (frontend only), PR 2 = `reconcile_sidecars_of` reporting
  + `ScanStarted` + the frontend `.then`.
- **Conflicts with in-flight work.** No edits to `crates/app/src/index.rs`
  (app-quick-fixes step 3) or `crates/app/src/shortcuts.rs` (the bug-fixes
  plan). `commands.rs` edits are confined to `reconcile_sidecars_of`,
  `ScanStarted`/`scan_folder` and the test module's sidecar tests.

## Progress

- (2026-09-20) Step 1 complete
