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

# Rating display tidy-up

## Purpose

Phase 6 shows a file's judgment in three places, each styled differently:
red upper-case `REJECTED` (or stars) drawn on the canvas, a small red `X`
(or stars) on the filmstrip cell, and yellow lower-case `rejected` (or stars)
under the file name in the meta pane next to the `N / M` position line. The
user finds the copies redundant and the styling inconsistent.

After this work the canvas shows only the image; the filmstrip keeps its
per-cell mark; the `N / M` counter (information about the list, not the
file) lives in the filmstrip pane; and the meta pane ends with a
rule-separated section headed by the sidecar's file name holding a `Rating`
row. The section exists because the EXIF rows are facts read from the ARW
that the app never writes, while the rating comes from the XMP sidecar that
the app and other tools write — a different kind of information that must
not look like one more EXIF row. It is also where a pick flag will go later,
so it is built as a small `<dl>` that takes a second row without
restructuring; nothing beyond `Rating` is added now (the pick flag waits on
the user bringing a DxO PhotoLab sidecar to see what it writes).

This is a display change. The rating logic (keys, optimistic update,
folder-token checks, the `touched` set, the writer) is untouched.

## Decisions taken before implementation

Settled with the user; do not reopen.

1. Main view: no rating / reject drawing on the canvas, in the fitted view or
   the 1:1 view.
2. Filmstrip: the per-cell mark (reject mark, stars, dimmed cell) stays.
3. `N / M` moves from the meta pane into the left (`#side`) pane, as a fixed
   line between the virtual list and the "Open folder" button, so the list's
   `scrollTop -> index` arithmetic is unaffected.
4. Meta pane: the `position` and `rating` lines under the file name go. At
   the bottom, after a rule, a section headed by the sidecar name (with a
   "not created" note when none exists) holds a `Rating` row: stars, the
   reject mark, or a dash when unrated.
5. The filmstrip and the meta section use the same color and the same
   symbol for stars and for a reject.

Defaults taken by the caller for the planner's open points (the user asked
for the work to go straight to a PR; any of these can be changed in review):

- Reject glyph: `✕` (U+2715) in both the strip and the meta row.
- Sidecar header name: the name the app would write (`FOO.xmp`), not the
  on-disk case variant; see Trade-offs.
- A `sidecar-error` does not revert the optimistic "has sidecar" flag.
- Transient notes stay above the sidecar section so the pane ends with it.
- Single-PR mode: the planner's two steps are folded into one.

### How the frontend learns whether a sidecar exists (investigated)

`ratings.xmp_size` / `xmp_mtime_ns` are the sidecar's stat as the app last
read or wrote it; `NULL` means no sidecar (`mark_written` stores `NULL` when
nothing was written, `reconcile_sidecars` nulls the stat of a clean row whose
sidecar is gone). So `xmp_size IS NOT NULL` is exactly "a sidecar was on disk
the last time the app looked", and `Index::entries` can return it as a
boolean with no schema change.

The sidecar's **actual file name is not stored anywhere**: `list_sidecars_in`
(`crates/app/src/commands.rs`) keys by lower-cased name and discards the
path, and `existing_sidecar` (`crates/app/src/sidecar.rs`) rediscovers a
case variant per write. Exposing the real name would need a new column (a
`SCHEMA_VERSION` bump, which `index.rs` says must first migrate or flush
dirty rows) or a `read_dir` per `folder_entries` call. Both are
disproportionate for a display change. The header therefore shows the name
the app would write, `FOO.xmp` (the rule of `riffle_core::xmp::sidecar_path`:
the ARW's file name with its extension replaced by `xmp`), and the README
says that a foreign `FOO.XMP` is shown as `FOO.xmp`. Nothing is written
under a different name than today.

After the app's own keypress the write is debounced (300 ms) and
`folder_entries` is not re-run by it; the `touched` guard already keeps a
scan-triggered refresh from undoing the optimistic rating. Sidecar existence
follows the same pattern: a keypress with a non-`null` rating marks the file
as having a sidecar at once (the writer creates one when none exists and
patches it otherwise), a `null` rating leaves the flag as it is (the writer
writes nothing on a sidecar-less file and `0` into an existing one), and the
flag is never turned off by the app. A `sidecar-error` is already reported
in the status line and does not revert the flag (see Trade-offs).

## Steps

- [x] Step 1: Expose `has_sidecar`, and move the rating display: canvas off, counter into the strip pane, sidecar section in the meta pane
  - Done when:
    - Rust (`crates/app/src/index.rs`):
      - `IndexedFile` has `pub has_sidecar: bool`, filled by
        `Index::entries` from `ratings.xmp_size IS NOT NULL` in the existing
        `LEFT JOIN`; no schema change, no `SCHEMA_VERSION` bump, no new
        command, no new IO in `folder_entries`
      - A test in `index.rs` shows the cases through `entries`: no `ratings`
        row -> `false`; a row after `set_rating` only (dirty, stat NULL) ->
        `false`; a row after `store_sidecar_ratings` or `mark_written` with a
        stat -> `true`; and `mark_written` with `None` -> `false` again
    - Frontend:
      - **(manual)** No rating / reject drawing on the canvas in the fitted or
        the 1:1 view; `drawRatingBadge` and its calls in `draw()` /
        `drawZoom()` are gone from `crates/app/ui/src/main.ts`
      - **(manual)** `N / M` appears in the left pane between the filmstrip
        and the "Open folder" button, updates on every page turn, is empty
        when no folder is open, and the strip still scrolls and virtualizes as
        before
      - **(manual)** The meta pane no longer shows a position or a rating line
        under the file name, and ends with a section separated from the EXIF
        rows by a rule, headed by the sidecar name (e.g. `_DSC0006.xmp`), with
        `(not created)` — or equivalent wording — appended when the file is
        known to have no sidecar, and a `Rating` row showing the stars, the
        reject mark, or a dash (`–`) when unrated
      - **(manual)** Pressing a rating key on a file without a sidecar makes
        the "not created" note disappear at once; `0` on such a file does not
      - **(manual)** The filmstrip cell and the meta `Rating` row use the same
        glyphs and the same colors (one yellow for stars, one red for the
        reject), and the reject mark is the same text in both places
    - `README.md`: the "Status" paragraph and the "Ratings and XMP sidecars"
      section say where the judgment is shown (strip cell and the meta
      pane's sidecar section; the canvas shows only the image; the counter is
      in the strip pane), say that the header shows the name the app would
      write so a foreign `FOO.XMP` reads as `FOO.xmp`, and the Phase 6
      "Awaiting the user's confirmation" paragraph no longer mentions a badge
      on the preview or the status line; a new "Awaiting the user's
      confirmation" paragraph lists the (manual) items above. The three-way
      split (confirmed by hand / verified without a GUI / awaiting) is kept
    - `mise run ci` passes (`tsc --noEmit` covers the frontend)
  - Implementation approach:
    - Rust: select the column as `ratings.xmp_size IS NOT NULL` (an integer
      0/1 in SQLite; read as `bool` like `thumb IS NOT NULL` already is). The
      `IndexedFile` doc comment gains one line saying what the flag means (the
      sidecar as the app last saw it, not a live stat). Do not touch
      `commands.rs` beyond what the compiler needs (nothing is expected:
      `folder_entries` returns the struct as is)
    - Add `has_sidecar: boolean` to the `IndexedFile` interface in `main.ts`
    - `crates/app/ui/index.html`: add `<div id="position"></div>` between
      `#strip` and `#open` inside `#side`. `style.css`: `#position { flex:
      none; ... }` in the same muted color the old `.position` line used, so
      `#strip` keeps `flex: 1` and `strip.ts` is not touched for this
    - `main.ts`, keep changes localized and additive (another session may be
      editing the file): a `positionEl` next to `metaEl` / `openEl`; set its
      `textContent` at the top of `renderMeta()` (called on every `show()`
      through `setStatus`), so no new call sites are needed. Remove the
      `position` and `rating` `line(...)` appends; remove `drawRatingBadge`
      and the two calls; drop the `draw()` calls in `rate()` that existed
      only to repaint the badge (the one in `refreshEntries` stays, it also
      serves the focus mark). Do not touch `rate()`'s map updates, the
      `touched` set, the token check or the revert path otherwise
    - Sidecar existence on the frontend: a `sidecars: Set<string>` beside
      `ratings` / `touched`, cleared in `openDirectory`, filled in
      `refreshEntries` for rows with `!touched.has(row.path)` (same guard as
      the rating), and added to in `rate()` when `rating !== null`. The
      header's note is three-state so the app does not claim "not created"
      before it knows: show the note only when `!sidecars.has(path) &&
      (entries.has(path) || touched.has(path))`; an entry that has not
      arrived yet (folder not scanned, no index cache) shows the name alone.
      Do not add a `pendingSidecars` set or a `sidecar-written` event
    - Sidecar name: a `sidecarName(path)` helper next to `baseName`, replacing
      the extension of `baseName(path)` with `xmp`, with a comment that it
      mirrors `riffle_core::xmp::sidecar_path` and that a case variant on disk
      is not known here
    - The section: after the EXIF `<dl>` (and after the transient `note` /
      `scanning` / `1:1` lines, so the section is always last — see
      Trade-offs), append a `<section class="sidecar">` containing a header
      `<div>` and a `<dl>` built with the existing `row()` helper:
      `row(list, "Rating", text)`. A later pick flag is one more `row()`
      call. The value element gets a class (`stars` / `rejected`) so CSS
      colors it; `row()` needs a small extension (an optional class on the
      `dd`) or a sibling helper — prefer the smallest change
    - Consistency: define `--stars-color: #ffd050` and `--reject-color:
      #ff6b6b` on `:root` in `style.css` and use them in `.cell span.rating`,
      `.cell.rejected span.rating` and the new meta rules; remove the
      `#meta .position` and `#meta .rating` rules and the `rgba`/hex
      literals that lived in `drawRatingBadge`. Use `✕` as the reject glyph
      in both `strip.ts` (`paintRating`) and the meta row; the stars stay
      `★`. A shared constant is not worth a new module; a comment in each
      file naming the other is enough
    - `strip.ts` changes are limited to the reject glyph and, if the color
      moves to a custom property, nothing else; the cell dimming stays
    - Write `learnings.md` as you go; if `tsc` or clippy surfaces anything
      about the `IS NOT NULL` column typing, record it there

## Trade-offs and risks

### Sidecar name: predicted versus actual

Taken: show the name the app would write (`FOO.xmp`) plus a boolean from the
index. A foreign `FOO.XMP` is therefore shown as `FOO.xmp` while the writer
patches `FOO.XMP`; the README says so. Alternatives, if the mismatch is
unacceptable: (a) a `read_dir` of the folder inside `folder_entries` to
attach the real name per row — live and exact, but adds a directory listing
(measured at ~36 ms for 5000 sidecars in Phase 6, on dummy files with a warm
cache) to a call that `scan-progress` can trigger up to 10/s while the
current file's row is missing, and makes `folder_entries` a disk read rather
than an index read; (b) a `xmp_name TEXT` column on `ratings` — exact and
cheap at read time, but a schema change that `index.rs` says must migrate or
flush dirty rows first, i.e. new migration machinery.

### Optimistic "has sidecar" and a failed write

Taken: a keypress with a rating marks the file as having a sidecar and a
`sidecar-error` does not revert it (the status line already reports the
error, and the writer retries on the next open, when the index says the
truth again). Alternative: revert on `sidecar-error` when the flag was set
optimistically, which needs a second set to remember which flags were
optimistic. Not taken for size.

### The reject glyph

The filmstrip writes `X`, the meta pane `rejected`, the canvas `REJECTED`.
Decision 5 wants one form in both surviving places. Taken: the glyph `✕` in
both — compact on a 144 px cell and symmetric with the stars being glyphs.
Alternative: the word `rejected` in both — self-explanatory in the meta row
but wordy on a thumbnail. Whichever is chosen, it is the same string in
`strip.ts` and `main.ts`.

### Where the transient notes go

The `note`, `scanning` and `1:1` lines are appended after the EXIF rows
today. The meta pane must *end* with the sidecar section, so the notes stay
where they are and the section comes after them. Alternative: notes below
the section. Minor; the implementer may swap if the result looks wrong, and
should note it in `learnings.md`.

### Unknown versus absent

Before the scan reaches a file (or with no index cache) there is no entry and
the app cannot know whether a sidecar exists. The header then shows the name
with no note rather than claiming "not created". This is the same window in
which the rating itself is unknown today, so nothing new is hidden.

### Parallel work in `main.ts`

Another session may be editing `crates/app/ui/src/main.ts`. The step removes
`drawRatingBadge` and two lines in `renderMeta`, and adds a helper, a set and
a section builder; it does not reorder or reformat anything else. Rebase onto
`origin/main` before the PR.

### Manual verification

GUI automation is denied on this machine, so every visual criterion is
**(manual)** and for the user. `mise run ci` (`tsc --noEmit`, clippy, tests)
is the only automated check; no step claims a visual check was done.

## Progress

- (2026-09-18) Step 1 complete
