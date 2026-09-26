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

# Write the RAW's EXIF Orientation into Riffle-made `.dop` sidecars

## Purpose

PhotoLab 10 displays an image by the `Orientation` key of the `.dop` item, not
by the RAW's EXIF, and treats a missing key as unrotated. The `.dop` files
Riffle mints (`template` in `crates/core/src/dop.rs`) and the Settings-less
ones it repairs (PR #470's `Doc::insert_settings`) carry no `Orientation`, so
every portrait shot judged in Riffle shows up sideways once PhotoLab loads the
sidecar. Verified in PhotoLab 10 on 2026-09-26: adding `Orientation = 8` (the
ARW's own IFD0 tag 0x0112 value) between `Name` and `Rating` fixes the display;
a foreign value (a sample's `8` on an EXIF-`1` file) rotates it wrongly, so the
value must be the file's own. After this work, fresh and repaired Riffle `.dop`
files carry the RAW's own EXIF Orientation and PhotoLab shows them upright.

## Current state (investigated)

- `crates/core/src/dop.rs`: `template()` writes `Name`, `Rating`, `Settings`,
  `ShouldProcess`, `Uuid` in that order; `Doc` has no `orientation` field;
  `locate()` collects item keys with `in_item(key)` at depth 4;
  `insert_settings` is the model for an "insert before an existing line, else
  before the item's closing brace" edit. Edits are applied via a stable
  `sort_by_key(Reverse(start))`, so same-offset inserts land in reverse push
  order.
- All PhotoLab fixtures (`crates/core/src/fixtures/dop/_DSC00{01..05,09..15}.ARW.dop`)
  already carry `Orientation = 8,` at depth 4, so "never touch an existing
  Orientation" keeps every byte-for-byte fixture test valid.
- `crates/app/src/sidecar.rs`: `SidecarFormat::write_rating` / `write_label`
  receive `arw: &Path`; `write_kind` calls them; `write` is called from
  `flush`, which holds the `Arc<Mutex<Index>>`. A test calls `dop::write_label`
  directly and needs the new argument.
- Cheap orientation sources: the index's `files.orientation` column
  (`Index::entry` returns `IndexedFile.orientation`, 1 when NULL), and
  `riffle_core::reader::read_metadata(path)` (1 MiB head, `arw::parse` reads
  IFD0 0x0112 and defaults to 1) for ARW and DNG alike.
- `docs/agents/tauri-app.md`: the `.dop` Settings subsection says not to copy an
  `Orientation` from a sample.

## Steps

- [x] Step 1: Write the RAW's EXIF Orientation into fresh and patched Riffle `.dop` items
  - Done when:
    - `dop::write_rating` and `dop::write_label` take an `orientation: Option<u16>` (next to `name`, from the caller) and:
      - the fresh template writes `Orientation = {n},` between `Name` and `Rating` when `Some(n)`, and no line at all when `None`;
      - patching an existing sidecar whose `Items[0]` has no `Orientation` key at the item's depth (4) inserts the same line at the start of the item's `Rating` line (between `Name` and `Rating` in Riffle-made files, including ones repaired by #470), falling back to before the item's closing brace when there is no `Rating` line; nothing is inserted when `orientation` is `None`;
      - an existing `Orientation` key (any value) is never spliced, moved or duplicated, whatever `orientation` is passed; the insert is idempotent across repeated writes.
    - `crates/app/src/sidecar.rs` supplies the orientation of the RAW being judged, read from the index's `files.orientation` row first and from `riffle_core::reader::read_metadata` when the file has no row yet; `None` when neither yields one. The index lock is released before the sidecar file I/O.
    - Tests in `crates/core/src/dop.rs`: the pinned template text gains the `Orientation` line and a `None` variant pins the line-less template; old-template tests show the line inserted once before `Rating` by both `write_rating` and `write_label`, idempotent on a second write; a sidecar with no `Rating` line gets it at the closing brace ahead of the other inserted keys (alphabetical); every PhotoLab fixture patched with `Some(1)` or `Some(6)` is byte-for-byte unchanged apart from the existing splices (its `Orientation = 8` survives).
    - Tests in `crates/app/src/sidecar.rs`: a fresh `.dop` minted for a RAW fixture carrying Orientation 8 contains `Orientation = 8,` between `Name` and `Rating`; the existing direct `dop::write_label` call is updated for the new signature.
    - `docs/agents/tauri-app.md`'s `.dop` Settings subsection and the `dop.rs` module doc state that Riffle writes the file's own EXIF Orientation between `Name` and `Rating`, why (PhotoLab uses the item's `Orientation` over the RAW's EXIF; missing = unrotated; foreign value = wrong rotation), the source of the value, the `None` fallback, and the push order at a shared offset.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/core/src/dop.rs`:
      - Keep the template in the single `template()` function; add an `orientation: Option<u16>` parameter and emit the `Orientation = {n},\n` line conditionally between `Name` and `Rating`.
      - Add `orientation: Option<Range>` to `Doc`, filled by `in_item("Orientation")` in `locate` (depth 4 only).
      - Add `Doc::insert_orientation(&self, text, orientation: Option<u16>)` modelled on `insert_settings`: nothing when the key exists or `orientation` is `None`; otherwise an insert at the start of the `Rating` line when it exists, else before the item's closing brace. Copy the indentation the way `insert_settings` does.
      - Same-offset ordering: later-pushed edits land in front. Push the orientation edit so a shared-offset insert comes out alphabetical (`ColorLabel`, `Orientation`, `Rating`, `Settings`, `ShouldProcess`); verify with exact-bytes tests; do not reorder the existing pushes.
    - `crates/app/src/sidecar.rs`:
      - Thread `orientation: Option<u16>` from `flush` into `write` / `write_kind` and into the `Dop` arm of `SidecarFormat::write_rating` / `write_label` (the `Xmp` arm ignores it).
      - In `flush`, look the orientation up in a short index-lock scope (`Index::entry`, or a small one-statement `Index::orientation` query); when there is no row, `riffle_core::reader::read_metadata(&path)`; on error, `None`. Do not hold the index lock across file reads or the sidecar write.
      - For the app-level test, reuse the existing `arw_with_preview(orientation, body)` TIFF-shell test helper (a copy or the smallest shared helper).
    - `docs/agents/tauri-app.md`: edit the existing subsection in place.

## Trade-offs and risks

- Unknown orientation: omit the line (chosen; consistent between template and
  patch, and an omitted line displays as unrotated anyway) vs. writing `1`.
  "Unknown" is rare: `arw::parse` defaults a missing tag to 1.
- Orientation source: index row first with a `read_metadata` fallback (chosen;
  dirty-row replays on folder open can touch hundreds of files) vs. always
  `read_metadata`.
- An explicit `Orientation = 1` line for landscape files has not been checked
  in PhotoLab in isolation; the user is checking one such file from the
  bulk-repaired folder.
- Signature change: `dop::write_rating` / `write_label` gain a parameter; the
  only callers are in `sidecar.rs`.

## Progress

- (2026-09-26) Step 1 complete
