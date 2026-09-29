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

# A clear message for HDR PQ (HEIF) CR3 files

## Purpose

A CR3 shot with HDR PQ on carries only HEVC images (`PRVW`, `THMB` and the
first track), so `cr3::parse` finds no JPEG and the reader fails with the
generic `no embedded preview`. In the app the strip shows a red cell with no
text and the viewer's note reads `\\?\D:\Photos\samples\CR3\R8.CR3: no
embedded preview`, which reads as a Riffle bug. After this work the reader,
the strip and the viewer say that the file is an HDR PQ (HEIF) CR3 whose
preview is not supported yet, other failures keep their messages, and the
docs and `todo.md` reflect that the clearer message is done while the HEVC
decoder (researched in a separate session) remains open.

## Steps

- [x] Step 1: Detect the HEVC-only CR3 in the core, report it through the
      reader, show it in the strip and the viewer, and update the docs
  - Done when:
    - `cr3::parse` marks a file whose images are HEVC (a `trak` whose `CRAW`
      sample entry carries an `HEVC` sub-box and no `trak` carries `JPEG`)
      and still returns its metadata (`CMT1` / `CMT2` / `CMT3`: EXIF, capture
      time, AF point) as today.
    - `reader::read_preview` / `read_full` on such a file fail with a
      specific message (proposed: `HDR PQ (HEIF) CR3: its HEVC preview is not
      supported yet`) instead of `no embedded preview`; a CR3 with no image
      at all, an ARW / NEF without a preview, and every other failure keep
      their current text.
    - The index hands the stored error text of a failed row to the
      frontend; the strip's failed cell carries it as its tooltip and shows
      a short visible text where the thumbnail would be; the viewer's note
      shows it (as today, prefixed by the path).
    - Unit tests: `cr3.rs` (the flag is set on a HEVC `trak` + non-JPEG
      `PRVW` / `THMB`, not set with only `CMP1` tracks, not set on a normal
      file), `reader.rs` (a temp CR3 built from `cr3::tests::Cr3` with a
      HEVC track and non-JPEG `PRVW` / `THMB` gives the specific message;
      `read_metadata` on it still returns the orientation / EXIF; a CR3
      without any track and preview still says `no embedded preview`),
      `index.rs` (`entries` exposes the error text of an error row and
      `None` for a good one). `mise run ci` passes.
    - Manual check on `D:\photos\samples\CR3\R8.CR3` and `R5m2.CR3` (never
      committed): the strip cell and the viewer show the message, the meta
      pane shows the EXIF rows, a star / flag / label writes a sidecar next
      to the file. Record the outcome in `learnings.md`.
    - `README.md` and `README.ja.md` (the HDR PQ sentence after the camera
      list), `docs/raw-formats.md` (the CR3 section's last sentence),
      `docs/agents/raw-metadata-parsing.md` (the "HDR PQ (HEIF) files carry
      no JPEG" section: name the flag and the message) and `todo.md` (the
      "Core: HDR PQ (HEIF) CR3 files cannot be opened" item: the clearer
      message is done, the HEVC decoder decision remains; note that error
      rows carry no metadata so bursts / the filter menu do not see these
      files) are updated. `docs/cameras.md` needs no change (the EOS R8
      stays unlisted; the limitation follows the setting, not the body).
  - Implementation approach:
    - Core, `crates/core/src/cr3.rs`: record the detection while walking
      the tracks in `parse` (the `jpeg_track` loop): a sample entry whose
      sub-boxes hold `HEVC` (the learnings in
      `../_archived/20260929-canon-nikon-raw/learnings.md` Step 3 list
      `HEVC` / `hvcC` / `GRID` on the R8 and R5 Mark II samples). Verify on
      the two local samples which sub-box names actually appear before
      fixing the check (a read-only walk with the CLI or a quick test
      binary is enough); the `PRVW` header's first u32 being 1 is a
      secondary signal, not the primary one. Keep `parse` returning `Ok`
      so `read_metadata` keeps working.
    - Core, `crates/core/src/arw.rs`: add one field to `Arw` (proposed
      `pub hevc: bool`, documented as "the embedded images are HEVC (a CR3
      shot with HDR PQ on), so `preview` and `full` are `None`"). The struct
      literals in `arw.rs`, `nef.rs`, `jpeg.rs` and `cr3.rs` set it (`false`
      except in `cr3`). This is the only way the reader learns the reason
      without re-walking the boxes.
    - Core, `crates/core/src/reader.rs`: in `embedded_from`, when
      `kind.pick(&arw)` is `None` and `arw.hevc`, return the specific
      message; keep it a `pub const` in `reader.rs` (or `cr3.rs`) so the
      tests assert on it. No path prefix: `scan::extract` stores
      `e.to_string()` in the index and `commands.rs` adds the path for the
      viewer, as today.
    - App, `crates/app/src/index.rs`: `INDEXED_FILE` selects `error`
      instead of `error IS NOT NULL`; `IndexedFile` gains `pub error:
      Option<String>` (serde, `None` for a good row) and `exif` stays
      `None` when it is `Some`. Update the test
      `a_failed_file_is_remembered_as_an_error_row_without_a_thumbnail`.
      Bump `EXTRACTOR_VERSION` to `8` with the doc comment line ("`8`
      names an HDR PQ (HEIF) CR3 in the error row") so already-indexed
      HDR PQ rows get the new text (decided by the user, 2026-09-30).
      If main has meanwhile moved `EXTRACTOR_VERSION` past 7 (other
      sessions add RAF / ORF), bump to the next free number instead.
    - App frontend, `crates/app/ui/src/main.ts` (`IndexedFile` gains
      `error: string | null`) and `crates/app/ui/src/strip.ts`: add a
      `setFailure(index, message | null)` next to `setSharpness` /
      `setBurst`, called where `setRating` is called on every entry refresh,
      painting `cell.el.title` with the full text and a `span.reason`
      (created in `createCell`) with the text; `crates/app/ui/style.css` next
      to `.cell.failed`: the reason is visible only on `.cell.failed`, small,
      wrapped and clamped to the cell. The `failed` set that toggles the
      class stays as it is. Do not special-case the HDR PQ string in the
      frontend: every failed file shows its stored text, which is what
      makes other failures keep their current messages.
    - Viewer: no change; `requestPreview`'s `setStatus(String(err))`
      already shows `<path>: <message>`.
    - MCP (`companion.ts`, `mcp.rs`): no change; the view state carries no
      error today.
    - CLI: no change; it goes through `reader` and gets the message for free.
    - Metadata for such files: `read_metadata` already parses them, so the
      meta pane works; confirm in the manual check. Storing the EXIF on an
      error row (so bursts, the filter menu and capture-time order see
      these files) changes `Entry` / `write_batch` and is not cheap; note
      it in `todo.md` rather than doing it here.

## Trade-offs and risks

- `EXTRACTOR_VERSION` bump: error rows are authoritative on reconcile, so
  without a bump a folder scanned before this change keeps `no embedded
  preview` for its HDR PQ files until the file itself changes. A bump
  re-extracts every folder once (thumbnails, sharpness) for every user.
  Decision (user, 2026-09-30): bump.
- Where the flag lives: a field on `Arw` (chosen; four struct literals
  change) versus a second box walk in `reader.rs` (`cr3::is_hevc(buf)`), which
  keeps `Arw` untouched but parses the file twice on the failure path. Or
  making `cr3::parse` itself fail, which is simplest but breaks
  `read_metadata` and so the meta pane for these files.
- Strip presentation: a visible short text in the failed cell (chosen: a
  tooltip alone is hard to discover) versus a tooltip only (smallest change,
  no CSS). Long generic errors (e.g. `CR3 box header past the buffer`)
  become visible in cells too; they are clamped, and the tooltip carries the
  full text.
- Detection precision: the `HEVC` sub-box on a track is the primary
  signal. If a sample turns out to carry only `hvcC` or the `PRVW` header
  version without an `HEVC` box, widen the check to those, still requiring
  that no `JPEG` track exists. A CR3 with neither (a truly empty or
  truncated file) must keep `no embedded preview`.
- Out of scope, left in `todo.md`: the HEVC decoder; EXIF on error rows so
  bursts and the filter menu see HDR PQ files.
- Other sessions are adding RAF / ORF parsers and may touch the `Arw`
  struct literals and `EXTRACTOR_VERSION`; expect a small conflict at
  merge time.

## Progress

- (none yet)
