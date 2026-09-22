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

# Shutter type in the meta pane

## Purpose

When culling, the user wants to see whether a Sony ARW was shot with the
mechanical shutter (with electronic front curtain) or the fully electronic
shutter, so that rolling-shutter or banding suspects can be told apart at a
glance. Sony bodies do not write a `ShutterType` tag on ILCE models, but the
MakerNote tag 0x201a (`ElectronicFrontCurtainShutter`, ExifTool 13.59) flips
with the shutter setting: `1` (On) on every mechanical-shutter shot and `0`
(Off) on electronic-shutter shots, verified across the 2026 library
(`/mnt/d/Photos/2026/2026-09-19` switches mid-day at `_DSC2450`). This work
surfaces that as a "Shutter type" row in the meta pane, Sony ARW only. Leica
DNG has no equivalent tag and shows no row.

## Steps

- [x] Step 1: Read Sony MakerNote tag 0x201a in core and show it as a "Shutter type" row in the meta pane
  - Done when:
    - `riffle_core::arw::Shot` carries the raw tag value (e.g.
      `pub electronic_front_curtain: Option<u32>` or an equivalent
      `Option<bool>`), `None` when the MakerNote is absent, non-Sony, or lacks
      the tag, with a unit test on synthetic bytes in `crates/core/src/arw.rs`
      covering value `1`, value `0` and the tag being absent.
    - `Metadata` in `crates/app/src/commands.rs` gains a
      `shutter_type: Option<String>` formatted as `"Mechanical"` for `1` and
      `"Electronic"` for `0`, `None` otherwise, with a code comment stating the
      mapping assumption (EFCS is always enabled on the user's bodies, so Off
      means electronic shutter; a fully mechanical shutter with EFCS disabled
      would also read Off).
    - The meta pane in `crates/app/ui/src/main.ts` shows a
      `row(list, "Shutter type", meta.shutter_type)` directly after the
      existing `"Shutter"` row, and the `Metadata` TypeScript interface mirrors
      the new field.
    - Opening a Leica DNG or an ARW without the tag shows no such row.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/core/src/arw.rs`:
      - Add `const TAG_ELECTRONIC_FRONT_CURTAIN_SHUTTER: u16 = 0x201a;` next
        to `TAG_FOCUS_MODE` (0x201b) with a doc comment. **The tag is written
        as TIFF type LONG (4), count 1, on the ILCE-7M5 files** (checked by
        walking the MakerNote IFD of `_DSC1796.ARW` → `type 4 count 1 value 1`
        and `_DSC2450.ARW` → `type 4 count 1 value 0`), so read it with the
        existing `integer()` helper (SHORT or LONG, count 1), not `byte()`.
      - Add the field to `Shot` (doc comment: "Raw Sony
        `ElectronicFrontCurtainShutter`: 1 on, 0 off") and populate it in
        `exif()` from the `maker` IFD the same way `focus_mode` /
        `af_tracking` are, i.e. `maker.as_ref().and_then(|m| m.iter().find(..)).and_then(integer)`.
        The MakerNote is already inside the bounded prefix
        `reader::read_metadata` reads, so nothing changes in `reader.rs`.
      - Tests: reuse the `tiff_with_sony_note` helper (line ~681) with an
        entry `(TAG_ELECTRONIC_FRONT_CURTAIN_SHUTTER, TYPE_LONG, 1, 1)` and
        `(.., 0)`; the absent case can reuse an existing note without the tag
        (e.g. as `a_maker_note_without_focus_location_is_none` does). The
        Leica helper `tiff_with_leica_maker_note` already asserts Sony fields
        stay `None`; extend or rely on that for the DNG case.
    - `crates/app/src/commands.rs`: extend `Metadata` and `read_metadata`
      following the `focus_distance` pattern (raw core value → display string
      in the command, no `exif.rs` change since the value is not used by the
      filter menu). Put the On→Mechanical / Off→Electronic caveat comment
      here where the mapping is made.
    - `crates/app/src/index.rs`: no change and no `SCHEMA_VERSION` bump. The
      meta pane reads the file live via `read_metadata`, and the `files`
      table only caches the fields the filter menu needs.
    - `crates/app/ui/src/main.ts`: add `shutter_type: string | null` to the
      `Metadata` interface (line ~75) and the `row(...)` call after
      `row(list, "Shutter", meta.shutter)` (line ~380).
    - Manual check before opening the PR: run the app on
      `/mnt/d/Photos/2026/2026-09-19`, confirm `_DSC1796` shows
      "Mechanical", `_DSC2450` shows "Electronic", and a Leica DNG shows no
      row.

## Trade-offs and risks

- Row label: "Shutter type" sits next to the existing "Shutter" (speed) row,
  matching ExifTool's naming for the concept.
- Mapping assumption: tag 0x201a is EFCS on/off, not shutter type. A fully
  mechanical shutter (EFCS disabled) would be shown as "Electronic". The user
  never disables EFCS, so the mapping is accepted and documented in a comment.
- Value type: only LONG count 1 was observed (ILCE-7M5); keep `integer()` only.
- Where to format: the string mapping lives in `commands.rs` (like
  `focus_distance`) rather than `exif.rs`, which stays limited to values shared
  with the filter menu.

## Progress

- (none yet)
