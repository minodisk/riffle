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

# Group the meta pane by provenance

## Purpose

The meta pane (`renderMeta()` in `crates/app/ui/src/main.ts`) shows one flat
`<dl>` mixing values read from standard EXIF/TIFF tags, values read from a
vendor MakerNote (Sony `ElectronicFrontCurtainShutter`, Leica
`FocusDistance`), and a score Riffle computes itself (Sharpness). A user
cannot tell which figures are what the camera wrote per the standard, which
are vendor-specific readings, and which are Riffle's own analysis.

The MakerNote is itself an EXIF tag (0x927C in the Exif IFD); only its
contents are vendor-defined. So the pane is split into two top-level groups,
**EXIF** and **Riffle**, and inside EXIF the MakerNote rows are set apart from
the standard rows by a thin divider with a small `Maker note` label. This
makes the provenance visible and gives future maker-specific and computed
rows (the Sigma BF AF point, more Sony AF fields) an obvious home.

Provenance of every current row, traced in `crates/core/src/arw.rs` (`exif()`),
`crates/app/src/exif.rs` and `crates/app/src/commands.rs` (`read_metadata`):

- EXIF, standard: Aperture (`FNumber` 0x829d, or `ApertureValue` 0x9202 as
  `(est.)`; both standard tags), Shutter (`ExposureTime`), ISO, Focal length,
  Exposure (`ExposureBiasValue`), Camera (IFD0 `Make` + `Model`), Lens
  (`LensModel` 0xA434 only, no MakerNote fallback), Captured
  (`DateTimeOriginal`).
- EXIF, maker note: Shutter type (Sony MakerNote 0x201a), Focus distance
  (Leica MakerNote 0x0304).
- Riffle: Sharpness (`crates/core/src/sharpness.rs`, delivered through the
  SQLite index into the frontend's `sharpness` map, not through the
  `metadata` invoke).

No row has a standard-or-MakerNote fallback, so the grouping is a static
table in the frontend and needs no Rust change.

## Layout

```
<file name>
EXIF
  Aperture      f/2.8
  ...
  Captured      ...
  ─ Maker note ─
  Shutter type  ...
Riffle
  Sharpness     303.5
```

## Steps

- [x] Step 1: Render the meta pane as EXIF (standard + maker note) and Riffle groups
  - Done when:
    - `crates/app/ui/src/meta.ts` exists, is DOM-free (pattern of
      `context.ts` / `empty.ts`), owns the `Metadata` interface moved out of
      `main.ts` (keep its "Mirrors `Metadata` in `crates/app/src/commands.rs`"
      comment), and exports a pure function, e.g.
      `metaGroups(meta: Metadata | null, sharpness: number | null): MetaGroup[]`
      where a `MetaGroup` is `{ heading: string; sections: { label: string | null; rows: { label: string; value: string }[] }[] }`.
      It returns, in order, the `EXIF` group (a standard section with
      `label: null`, then a `Maker note` section) and the `Riffle` group (one
      section with `label: null`). Rows whose value is `null` are dropped,
      sections with no rows are dropped, groups with no sections are dropped.
    - The Riffle group is built from the `sharpness` argument only, so a
      file whose `metadata` invoke failed (`meta === null`) still shows its
      score. The `toFixed(1)` formatting moves into `meta.ts` with it.
    - `crates/app/ui/src/meta.test.ts` (vitest, style of `context.test.ts`)
      covers: Sony-like input yields EXIF (standard rows in the existing
      order, then a Maker note section with Shutter type) and Riffle
      (Sharpness); Leica-like input puts Focus distance in the Maker note
      section; null rows are dropped; input with neither `shutter_type` nor
      `focus_distance` yields no Maker note section; `meta === null` with a
      score yields only the Riffle group; `sharpness === null` yields no
      Riffle group.
    - `renderMeta()` in `main.ts` replaces the `row(...)` calls with a loop
      over `metaGroups(...)` that appends, per group, a small heading and,
      per section, an optional divider-with-label followed by a `<dl>` of its
      rows. `row()` and `line()` stay in `main.ts`. The `main.ts` diff is
      confined to the `Metadata` interface removal + import and the body of
      the `if (meta !== null)` block (which no longer gates the Riffle group).
    - `crates/app/ui/style.css` gains rules next to `#meta dl` (around line
      569): group heading small and muted (`#999`, the existing `dt` color);
      the Maker note label smaller still, with a thin top border as the
      divider; no new palette entries. The first heading carries the existing
      `0.75rem` top margin so the pane's vertical rhythm is unchanged.
    - `docs/usage.md` line ~51 (Meta pane bullet) describes the EXIF group
      with its Maker note section and the Riffle group; line ~131 ("The meta
      pane shows the raw score.") names the Riffle group. The `CLAUDE.md`
      layout sentence ("its `src/context.ts` builds ...") also names
      `src/meta.ts` as the grouping of the meta pane rows.
    - `mise run ci` passes.
  - Implementation approach:
    - Heading strings `EXIF`, `Maker note`, `Riffle` are exported constants
      in `meta.ts` so tests use the same strings.
    - The `metaStale` behavior is unchanged: stale rows keep showing the
      previous file's `meta` while the sharpness row already follows the
      current `files[index]`; do not "fix" this here.
    - Do not touch `crates/app/src/commands.rs`, `exif.rs`, or anything under
      `crates/core`.
    - Do not add a focus point / AF row in this step.
    - The parallel `feature/sigma-bf-af-point` branch changes only
      `crates/core/src/arw.rs`, README, CHANGELOG, todo.md and its plan folder;
      none of this plan's files. Keep `renderMeta()` edits local anyway.

## Trade-offs and risks

- **Nested headings vs. flat.** The user chose EXIF / Riffle as the top
  level, with the MakerNote rows separated inside EXIF by a labeled divider
  rather than a second heading level, since the MakerNote is part of EXIF and
  two heading levels are heavy in a narrow pane.
- **Grouping rule for fallback fields.** No displayed field falls back from a
  standard tag to a MakerNote, so a static per-field table is exact. If a
  future field gains such a fallback, revisit (a per-field source flag from
  `commands.rs`).
- **Sigma BF AF point, once shown.** Its x, y come from the MakerNote, but its
  frame (1000x667) is Riffle's inference. Recommended placement: Maker note,
  shown without the inferred frame size. Not implemented here.
- **`meta === null` now still renders the Riffle group.** The pane reads
  "name, Riffle: Sharpness, error". Intended; note it in the PR.
- **Heading margins change the pane's rhythm.** Verify with a Sony and a
  Leica file, and in the `metaStale` state while paging.

## Progress

- (2026-09-24) Step 1 complete
