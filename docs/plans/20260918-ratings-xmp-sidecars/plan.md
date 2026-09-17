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

# Phase 6: ratings, reject flag, and XMP sidecars

## Purpose

The app exists to page through ~5000 ARW files from a game and record a
judgement on each. Up to Phase 3 it can only look. Phase 6 is where a
judgement is finally recorded: `1`-`5` set a star rating, `x` marks a reject,
and the result is written to a standard XMP sidecar (`FOO.ARW` ->
`FOO.xmp`) next to the RAW so Lightroom, Bridge, Capture One and darktable
read it back. The RAW file is never touched.

The product's top priority is responsiveness under key-mashing: a keypress is
reflected on screen at once and never waits on disk. So the sidecar write is
asynchronous, coalesced per file and made durable through the SQLite index
(which already exists and was planned to carry ratings), and the filmstrip and
the main view show the current rating / reject state so the user can tell
what they have already judged.

Scope guard: no filtering by rating (a later phase; this phase only stores
what it would filter on), no prefetch (Phase 4), no 1:1 focus check (Phase
5), no burst grouping (Phase 7), no pick flag (`p`), no colour labels, and
nothing beyond `xmp:Rating` is written to the sidecar.

## Decisions taken before implementation

Settled with the user; do not reopen. The constraints from the brief (no RAW
decoder, the RAW is never written or renamed, `tsc`-only frontend with
`./foo.js` imports, IO commands are `async` + `spawn_blocking`, everything in
English, `mise run ci` is the only CI, **no subprocess**) are settled and not
repeated here.

1. **A reject is `xmp:Rating="-1"`; nothing else is written.** Evidence: the
   exiftool XMP tag reference documents `xmp:Rating` as "a value from 0 to
   5, or -1 for 'rejected'", which is the convention Adobe Bridge writes for
   a rejected file, and darktable both reads and writes. Lightroom Classic
   does not write pick/reject flags to XMP at all (flags are catalog-only)
   but reads `-1` as a reject on import, so the app's rejects survive a trip
   into Lightroom while Lightroom's own rejects can never reach a sidecar
   from any tool. `xmp:Label` is not used: it is a *colour label* in every
   reader, so writing `Label="Rejected"` would put a nonsense colour on the
   file rather than a reject. A private namespace would be read by nobody.
   Consequence: the model is one signed integer per file, `-1..5`, `None`
   when unrated; a reject and a star rating cannot coexist (see 5).
   **The user's downstream tools are Lightroom and DxO PhotoLab.** Whether
   PhotoLab reads `-1` as a reject could not be verified during planning (the
   web checks returned 403); the user was told so, was offered a belt-and-
   braces `xmp:Label="Red"` alongside `-1`, and chose `-1` alone. If PhotoLab
   turns out to ignore it, adding the label is a one-line change in
   `write_rating`. **(manual)**: which tools read `-1` back is the user's to
   confirm with what they have installed.
2. **The sidecar is the source of truth; the index is a cache plus a
   write-ahead buffer.** Interoperability is the whole point of the sidecar,
   and the index is already documented as a discardable cache. Concretely, a
   new `ratings` table (independent of `files`, so a rescan of a changed ARW
   or a failed extraction never drops a rating) holds per path: `rating`
   (`-1..5` or NULL), `xmp_size` / `xmp_mtime_ns` of the sidecar as last
   read or written by the app (NULL = no sidecar seen), and `dirty` (1 while
   an app edit has not yet landed in the sidecar). Reconciliation on every
   folder open, after the existing `reconcile`, stats each file's sidecar:
   - sidecar present, `(size, mtime)` differ from the stored pair, row not
     dirty -> parse it, store its rating and stat (external edit wins; this
     is also the first-open-of-a-Lightroom-folder case, where there is no
     row at all);
   - sidecar present, differs, row **dirty** -> the sidecar still wins and
     `dirty` is cleared. Reasoning: an app edit that never reached disk
     followed by an external edit is the rarer sequence, and "a sidecar that
     changed under us is read, never silently overwritten" is the rule a
     user can predict; the lost keypress is one file, visible on screen;
   - sidecar present, same `(size, mtime)` -> nothing (fast path for 5000
     files);
   - sidecar absent, row dirty -> write it now (the crash-recovery path);
   - sidecar absent, row not dirty, stored stat not NULL -> the sidecar was
     deleted externally: clear `rating` and the stat (the truth is gone);
   - sidecar absent, no row or stored stat NULL -> nothing.
   Parsing only sidecars whose stat changed keeps the second open at "stat
   5000 more files", which Phase 3 measured at ~40 ms for the ARWs
   themselves.
3. **Coalescing and durability.** On a keypress the frontend updates its own
   state and redraws first, then invokes `set_rating(path, rating)` and does
   not await it for anything visible. `set_rating` (async, `spawn_blocking`)
   writes the `ratings` row with `dirty = 1` in one small transaction
   (`PRAGMA synchronous = NORMAL` under WAL: a commit is a WAL append, no
   fsync) and hands `(path, rating)` to a single writer thread over a
   channel. The writer keeps a per-path pending map with a **300 ms
   trailing debounce** (mashing `1`,`2`,`3` on one file writes once, with
   `3`), writes the sidecar to `FOO.xmp.riffle-tmp` in the same directory,
   `fsync`s, renames it over `FOO.xmp` (atomic on APFS and NTFS; a crash
   mid-write leaves a stale but well-formed sidecar plus a temp file, never a
   truncated sidecar), then clears `dirty` only if the row's rating is still
   the value it wrote. On quit, `tauri::RunEvent::ExitRequested` drains the
   writer synchronously with a bounded wait (about 2 s) before the process
   ends; a normal Cmd+Q therefore loses nothing. Failure modes, stated
   plainly: a kill -9 or a crash inside the debounce window loses the
   sidecar write but not the judgement (the row is dirty and is written on
   the next open of that folder, rule 2); a keypress is lost only if the
   process dies between the DOM event and the `ratings` insert (a few ms);
   a sidecar directory that is read-only (an SD card, a locked share) fails
   the rename, the row stays dirty, the status line shows one message and
   the rating stays in the index until a later open can write it; a
   discarded index (schema bump, cache deleted) loses only the dirty window
   of rows not yet written, since everything else is in the sidecars.
4. **No XMP crate for writing; `quick-xml` for reading, which is already in
   `Cargo.lock` (0.42, pulled in by Tauri) so it costs no new dependency.**
   A fresh sidecar is a fixed ~10-line RDF/XML template with the `xpacket`
   wrapper Adobe expects, written with `format!`. The hard half is a sidecar
   someone else wrote: a Lightroom sidecar carries kilobytes of `crs:`
   develop settings, and the app must not destroy them. So an existing
   sidecar is **patched by byte splicing, not regenerated**: `quick-xml`'s
   reader (with `buffer_position()`) locates the `Rating` property of the
   `http://ns.adobe.com/xap/1.0/` namespace under the first
   `rdf:Description`, in either of its two legal shapes (attribute
   `xmp:Rating="3"`, or element `<xmp:Rating>3</xmp:Rating>`), and the app
   replaces exactly that value's bytes; when the property is absent it
   inserts one attribute into the first `rdf:Description` start tag;
   everything else stays byte-identical, which a test asserts. Namespace
   prefixes must be resolved (`xap:` and a default namespace are both seen
   in the wild), which is why a regex is not enough and quick-xml's
   `NsReader` is used. A file that is not parseable XML or has no
   `rdf:Description` is left alone and reported, never overwritten. The
   `xmp_toolkit` crate (Adobe's C++ SDK through FFI) is rejected for the
   C++ build on both platforms; `roxmltree` would work equally well but is
   a new dependency for no gain over what is already compiled.
5. **`x` is sticky, not a toggle; `u` un-rejects; `0` clears the rating; a
   rating and a reject replace each other.** A user mashing keys must get
   idempotent keys: `3` twice is 3 stars, `x` twice is a reject, never
   "cleared by accident". `x` sits between `s` and `d` (noted in the Phase 3
   plan), so a mistyped reject needs a cheap undo — that is `u`, which is
   also Lightroom's unflag key. `0` clears a star rating. Because the model
   is one field (decision 1), `x` on a 4-star file makes it `-1` (the star is
   gone, as in Bridge), and `1`-`5` on a rejected file un-rejects it and
   rates it. Both `0` and `u` write the "unrated" state, which is
   `xmp:Rating="0"` when a sidecar already exists (so a foreign sidecar's
   stars are cleared rather than left) and *no sidecar at all* when none
   exists (the app does not litter a folder with 5000 empty sidecars).
   Collision check: `0` and `u` are claimed by nothing (`Space`, `o`, `f`,
   the paging letters and `p` for a future pick flag are the reserved set).

## Steps

- [x] Step 1: Record the user's Phase 3 confirmations in the README
  - Done when:
    - `README.md` "What has been confirmed, and by what" moves the five
      items the user has now confirmed by hand — the `scanning N / M`
      progress line and responsiveness while a real folder scans;
      thumbnails filling in during a scan and portrait cells upright; the
      focus box landing on the subject; a fast second open; folder and
      single-ARW drag-and-drop — into a "Confirmed by hand on macOS
      (Phase 3)" paragraph, and the "Awaiting the user's confirmation
      (Phase 3)" paragraph is removed or reduced to whatever the user did
      not confirm (the Phase 3 keys and `f` are implied by the above)
    - The todo item "App: README's 'Awaiting the user's confirmation' list
      has to be updated by hand" in `todo.md` is removed (this step closes
      it). The item "App: real-folder scan and second-open numbers are still
      missing" stays: "fast" is a confirmation, not a measurement
    - `mise run ci` passes
  - Implementation approach:
    - Documentation only; touch nothing under `crates/`. Keep the
      three-way split (confirmed / verified without a GUI / awaiting) that
      `docs/agents/tauri-app.md` asks for

- [ ] Step 2: Core: XMP sidecar read, write and patch (`riffle_core::xmp`)
  - Done when:
    - `crates/core/src/xmp.rs` exposes: `sidecar_path(arw: &Path) ->
      PathBuf` (replace the extension with `xmp`; when the directory already
      has a sidecar differing only in case, e.g. `FOO.XMP`, prefer the
      existing one — resolved by the caller with a directory listing, so
      the function itself is pure); `read_rating(bytes: &[u8]) ->
      Result<Option<i8>, String>` returning the `xmp:Rating` of the first
      `rdf:Description` that has one (`None` when absent, `Err` when the
      bytes are not XMP, clamped to `-1..=5` with anything outside treated as
      absent); `write_rating(existing: Option<&[u8]>, rating: Option<i8>) ->
      Result<Vec<u8>, String>` producing a fresh template when `existing` is
      `None`, and otherwise the existing bytes with only the `Rating` value
      replaced or one attribute inserted (decision 4). `rating = None` on an
      existing sidecar writes `0`; on no sidecar the caller must not write a
      file at all (state this in the doc comment and enforce it in Step 3)
    - Both property shapes (attribute and element) and both prefixes (`xmp`,
      `xap`) are handled, namespace-resolved, and the `x:xmpmeta` /
      `xpacket` wrappers are optional on read
    - Tests with in-test fixtures (nothing derived from the user's files):
      a Bridge-style attribute sidecar; a Lightroom-style element sidecar
      with a large `crs:` block and a `dc:subject` bag, whose bytes outside
      the spliced value are asserted equal before and after; a sidecar with
      no `Rating` gets exactly one attribute inserted; `-1` round-trips;
      `xap:` prefix; a default-namespace `rdf:Description`; a truncated
      document is `Err`; `read_rating(write_rating(None, Some(n)))` round
      trips for every `n` in `-1..=5`; and the fresh template is byte-for-
      byte the documented one. `mise run ci` passes
  - Implementation approach:
    - `quick-xml = "0.42"` added to `crates/core/Cargo.toml` (same version as
      `Cargo.lock` already has, so no second copy is built). Use `NsReader`
      for namespace resolution; take byte ranges from `buffer_position()`
      before and after each event to locate the attribute value / element
      text to splice. Verify during implementation that the positions are
      exact for the attribute case; if quick-xml does not expose attribute
      value offsets, locate the attribute within the start tag's byte range
      (which is exact) with a small scan of that range only
    - Template for a fresh sidecar: an `<?xpacket begin=...?>` header with the
      standard `W5M0MpCehiHzreSzNTczkc9d` id, `<x:xmpmeta xmlns:x="adobe:ns:meta/"
      x:xmptk="riffle">`, an `<rdf:RDF>` with one `<rdf:Description rdf:about=""
      xmlns:xmp="http://ns.adobe.com/xap/1.0/" xmp:Rating="N"/>`, then
      `</x:xmpmeta>` and `<?xpacket end="w"?>`. No padding block (Adobe pads
      in-file XMP; sidecars do not need it). UTF-8, no BOM
    - Core stays free of SQLite and Tauri; nothing here knows about the index

- [ ] Step 3: App: ratings in the index, the `set_rating` command, and the coalesced sidecar writer
  - Done when:
    - `crates/app/src/index.rs`: `SCHEMA_VERSION` becomes 2 (the cache is
      rebuilt; nothing in it is lost that a scan cannot redo — the last
      time this is true, see the comment below) and adds `ratings(path TEXT
      PRIMARY KEY, dir TEXT NOT NULL, rating INTEGER, xmp_size INTEGER,
      xmp_mtime_ns INTEGER, dirty INTEGER NOT NULL DEFAULT 0)` with an index
      on `(dir)`. `PRAGMA synchronous = NORMAL` is set next to `journal_mode`.
      A comment on `SCHEMA_VERSION` records that from v2 on a bump discards
      pending (dirty) sidecar writes and must be paired with a migration or
      a flush; `entries` returns `rating: Option<i8>` per row through a
      `LEFT JOIN ratings USING (path)`, and `IndexedFile` gains the field
    - `Index::set_rating(dir, path, rating) -> Result<(), String>` upserts
      the row with `dirty = 1`; `Index::mark_written(path, rating, size,
      mtime_ns)` clears `dirty` and stores the stat **only if** the row's
      rating still equals `rating`; `Index::dirty_rows(dir)` lists what still
      needs writing
    - `crates/app/src/sidecar.rs` (new): a writer thread owning
      `HashMap<PathBuf, (Option<i8>, Instant)>`, fed by a
      `std::sync::mpsc` channel from `set_rating`, that flushes each entry
      300 ms after its last update: read the existing sidecar if present
      (case-insensitive match in the directory), `xmp::write_rating`, write
      `FOO.xmp.riffle-tmp`, `fsync`, rename over the sidecar, `stat` the
      result and `mark_written`. `None` with no existing sidecar writes
      nothing and marks written with NULL stat. Errors are sent as a
      `sidecar-error { path, message }` event and leave `dirty` set
    - `set_rating(path, rating)` is an `async` command doing the SQLite
      write under `spawn_blocking` and then `send`ing to the writer;
      `rating` outside `-1..=5` is an `Err`
    - Quit: `tauri::Builder::build(...).run(|app, event| ...)` handles
      `RunEvent::ExitRequested` by sending a `Flush` message and waiting on a
      reply channel for at most ~2 s, then lets the exit proceed; the
      `Drop` of the writer is not relied on
    - Tests in `crates/app` against a temp dir and temp database: a burst of
      three `set_rating` calls within the debounce window produces exactly
      one sidecar containing the last value; a sidecar that already exists
      (from Step 2's Lightroom fixture) is patched, not replaced; no temp
      file is left after a successful write; `mark_written` does not clear
      `dirty` when the rating changed in between; `dirty_rows` reports a
      row whose write failed (simulate with a read-only directory on Unix,
      skipped on Windows); a `Flush` message writes pending entries before
      the 300 ms elapse. `mise run ci` passes
  - Implementation approach:
    - Keep the writer a plain `std::thread` with `recv_timeout` against the
      earliest deadline rather than a tokio task: nothing in it is async,
      and the drain on exit is then a synchronous handshake. Send the
      `AppHandle` in for the error event (`Emitter` is safe from any thread,
      as `run_scan` already relies on)
    - The `ratings` row is written before the channel send so a crash after
      the send but before the sidecar still has `dirty = 1`
    - The scan's `write_batch` / `reconcile` must not touch `ratings`; a
      test guards that a `reconcile` that deletes a `files` row leaves the
      `ratings` row alone
    - Assumes Step 2 is merged

- [ ] Step 4: App: reconcile sidecars on folder open, and flush dirty rows
  - Done when:
    - `Index::reconcile_sidecars(dir, files)` applies the six rules of
      decision 2 in one transaction, returning the paths whose sidecar has to
      be parsed (its bytes are read and parsed outside the lock, on the
      blocking task, then stored) and the dirty paths to hand to the writer.
      Sidecar stats are collected in `scan_folder` by one directory listing
      of `dir` (already done for the ARWs; extend `list_arw_in` or read the
      directory once more and match `*.xmp` case-insensitively) rather than
      5000 individual `stat` calls of guessed names
    - `scan_folder` runs it after the existing `reconcile`, before
      returning; `folder_entries` then reflects Lightroom-set ratings on a
      folder the app has never seen. Dirty rows found on open are pushed to
      the writer with a zero debounce
    - Measured and recorded in `learnings.md` and the README: the second
      open of the 5000-symlink folder with and without 5000 sidecars
      present (a scratch folder of generated sidecars), so the cost of the
      sidecar pass is visible next to Phase 3's 34.4 ms; state the same
      caveats (symlinks, warm cache)
    - Tests: each of the six rules against a temp dir (external edit wins
      over a dirty row; deleted sidecar clears the rating; unchanged stat
      parses nothing — assert by counting parsed paths; first open of a
      folder with foreign sidecars populates ratings without any `files`
      row). `mise run ci` passes
  - Implementation approach:
    - The sidecar parse happens on the blocking task, bounded: read at most
      e.g. 4 MiB per sidecar (a Lightroom sidecar is tens of KB) and treat
      an oversize or unparseable one as "unknown" (leave the row as is,
      count it, do not error the open)
    - Assumes Step 3 is merged

- [ ] Step 5: Frontend: rating keys, optimistic state, and rating badges in the main view and the strip
  - Done when:
    - `crates/app/ui/src/main.ts` `keydown`: `1`-`5` set that rating, `x`
      sets `-1`, `u` clears only if the current value is `-1`, `0` clears
      any value; all are idempotent (pressing the current value again does
      nothing, not even an invoke). Modifier-held keys are still ignored;
      handled keys `preventDefault()`. The handler updates a local
      `ratings: Map<path, number | null>`, redraws, then invokes
      `set_rating`; the invoke is not awaited before the redraw. A rejected
      invoke reverts the map entry and shows the error in the status line.
      The result is checked against the folder token per
      `docs/agents/tauri-app.md` (no new counter)
    - The main view shows the current file's state in the status line
      (stars / `rejected` / nothing) and as a small overlay in a corner
      of the canvas drawn in `draw()` (so it is present during key
      auto-repeat and after a resize); the strip shows the same on each
      cell (stars or an `X` in a corner, rejected cells dimmed), with the
      state read from the shared `ratings` map, refreshed from
      `folder_entries` on open and on `scan-done` without overwriting
      entries the user changed since (the map is authoritative for anything
      set through the keyboard in this session until the next folder open)
    - A `sidecar-error` event shows its message once in the status line
    - `README.md` key table gains `1`-`5`, `x`, `u`, `0`
    - `mise run ci` passes. **(manual)** The user confirms: a rating key
      changes the badge and the status line with no perceptible delay,
      holding `3` down does nothing beyond the first press, `x` then `u`
      then `4` gives four stars, mashing keys while paging never shows a
      badge on the wrong file, the strip badge matches the main view, and
      `ls` in the folder shows one `.xmp` per rated file appearing within
      about half a second
  - Implementation approach:
    - `strip.ts` gets an exported `setRating(index, rating)` and reads the
      value at `createCell` / `highlight`; keep the DOM per cell to one
      extra `<span>` so the virtual list's cost is unchanged. Collision
      check against reserved keys (`Space`, `o`, `f`, `p`, paging keys):
      `0`, `1`-`5`, `u`, `x` are free — record it in the PR description
    - `IndexedFile` in `main.ts` gains `rating: number | null`
    - Assumes Step 4 is merged (for the initial values); the key handling
      itself only needs Step 3

- [ ] Step 6: Documentation and status update
  - Done when:
    - `README.md` "Status" says Phase 6 is done, documents the sidecar
      (`FOO.xmp`, `xmp:Rating` with `-1` for a reject, that foreign
      sidecars are patched in place and never regenerated, that the sidecar
      is the truth and the index a cache with a write-ahead role, the six
      reconciliation rules in prose, the 300 ms coalescing and the
      write-to-temp-and-rename, and what a kill during the debounce window
      does), lists which tools are *reported* to read `-1` and which the
      user has actually confirmed, and carries the Step 4 measurement
    - `CLAUDE.md` "Layout" mentions `crates/core/src/xmp.rs` and
      `crates/app/src/sidecar.rs`
    - `docs/agents/tauri-app.md` gains any pitfall Steps 3-5 hit (the
      `RunEvent::ExitRequested` drain is a likely candidate)
    - `mise run ci` passes

## Trade-offs and risks

### Reject representation (decision 1)

Taken: `xmp:Rating="-1"`. Alternative: also write `xmp:Label`. Not taken
because `Label` is a colour in every reader, and because the user declined it
when offered. Risk: a reader that does not understand `-1` shows a rejected
file as unrated; Lightroom's own rejects never reach a sidecar, so a round
trip through Lightroom cannot bring a reject back — both are properties of
those tools, not of this choice. The user should verify with Lightroom and
DxO PhotoLab **(manual)**; if PhotoLab ignores `-1`, adding a colour label is
a one-line change in `write_rating`.

### Source of truth (decision 2)

Taken: sidecar wins on every conflict, including over a dirty row.
Alternative: the dirty row wins (the app's unwritten keypress is re-applied
over an external edit). It protects the app's own edit but makes an external
edit silently disappear, and the sidecar exists for external tools. A third
option, prompting, is ruled out by the responsiveness goal and by "no
questions during open". If the user disagrees, only `reconcile_sidecars`'s
dirty branch changes.

### Where "unrated" lands

`0` / `u` on a file without a sidecar writes nothing rather than an empty
sidecar. A user who rates everything then clears some will therefore have
sidecars only on the files that were ever rated; harmless, but different
from "every judged file has a sidecar". Alternative: always write. Not taken
because it litters a folder that was previously clean.

### Toggle versus sticky (decision 5)

Taken: sticky with explicit clears (`u`, `0`). Alternative: `x` toggles, as
in some cullers. Toggle is one key fewer but is exactly the behaviour that
turns a double-press under auto-repeat into an accidental un-reject.

### Schema bump

`SCHEMA_VERSION` 1 -> 2 discards the existing cache on first launch after
Step 3 (one rescan, measured at 5.55 s for 5000 files; the README already
says the index is a cache). From v2 onward a bump can drop dirty rows, so the
comment in Step 3 asks future phases to migrate or flush first.

### `quick-xml` positional splicing

The design depends on exact byte positions from quick-xml's reader. If they
turn out not to be exact for attribute values, Step 2 falls back to locating
the attribute inside the start tag's range, which is exact. The worst case is
a full parse-and-rewrite through quick-xml's writer, which loses formatting
but not content; it is not the plan, and a test asserts byte identity outside
the spliced value so a regression is caught.

### Manual verification

GUI automation is impossible on this machine (`osascript` assistive access
is denied), so every criterion needing a running app is marked **(manual)**
and is for the user to confirm. This phase has an unusually high share of
them, since the whole point is a key-to-pixel loop: the responsiveness
claims in Step 5 cannot be measured here at all; only the Rust-side costs
(Step 4's reconcile time, the writer's coalescing) are covered by tests and
measurements. Cross-tool reading of the sidecars (Lightroom, DxO PhotoLab) is
likewise the user's to confirm.

### Parallel work

Phase 5 (the 1:1 focus check) is being implemented at the same time in a
separate worktree. It has been told not to touch `index.rs`'s schema, which
Step 3 bumps; `Space` is its key and `0`, `1`-`5`, `u`, `x` are this phase's.
Both edit `crates/app/ui/src/main.ts`, so changes here stay additive and
localised, and each PR rebases onto `origin/main` before merging.

### Todo items this phase touches

Closes "App: README's 'Awaiting the user's confirmation' list has to be
updated by hand" (Step 1). Leaves "App: no rescan when new files appear in an
already-open folder" as is, but note that the same watcher would also pick up
sidecars edited while the folder is open — today an external edit is seen
only on the next open. Does not touch the `ping`, `VACUUM`, or strip-refresh
items.

## Progress

- (2026-09-18) Step 1 complete
