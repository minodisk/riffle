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

# Minimal `Settings` block in Riffle-made `.dop` sidecars

## Purpose

A `.dop` that Riffle creates from scratch (the `template` function in
`crates/core/src/dop.rs`) has no `Settings` table under
`Sidecar.Source.Items[0]`. DxO PhotoLab 10 rejects such a sidecar: the folder
shows "no images in this folder", so the pick / reject / rating Riffle wrote
never appear. The user bisected this by hand in PhotoLab 10 on 2026-09-26:

- A CRLF tail, `ProcessingStatus = 1`, `Software = "DxO PhotoLab 10.0.1"`,
  `Albums` / `IPTC` / `Keywords` / `OutputItems`, or `CafId` / `ShotDate`
  alone do NOT make the image load.
- A brace-balanced `Settings` block does, with the pick (`ShouldProcess = 0`)
  shown and the rotation correct. The minimal accepted shape is
  `Settings = {\nVersion = "21.0",\n}\n,\n` in `Items[0]`, placed between
  `Rating` and `ShouldProcess` (PhotoLab writes the item's keys in
  alphabetical order). `Settings = { Base = {}, Overrides = {}, Version =
  "21.0" }` and PhotoLab's full block also work. A preset
  (`AppliedPresetDisplayName` ...) or an `Orientation` copied from a sample
  must NOT be added: the preset would override the user's default preset and
  a foreign `Orientation` rotated the image.

The archived learnings
`docs/plans/_archived/20260918-photolab-dop-sidecar/learnings.md` (Step 5)
claim "The minimal template (no `Settings` block) was accepted by PhotoLab
10". That was wrong (most likely the folder was already in PhotoLab's
database at the time).

Once done, a fresh Riffle `.dop` loads in PhotoLab 10 with its judgment, and
the Settings-less `.dop` files Riffle wrote so far (about 343 in one user
folder) are repaired the next time Riffle writes to them, with no bulk
migration command.

## Current state (investigated)

- `crates/core/src/dop.rs`: `template(rating, flag, name, now, uuids)` is the
  one fresh template (LF line endings, no indentation, `}\n,\n` after each
  nested table). `write_rating` / `write_label` with `existing = Some(..)`
  call `locate` (brace-depth `scan` + `field` at the item's depth 4), then
  build a list of `(start, end, replacement)` edits through `Doc::edit`, which
  splices a found value or inserts a `{indent}{key} = {value},\n` line just
  before the item's closing brace (`doc.item_close`) when the key is absent.
  Edits are sorted by start descending and applied with `replace_range`;
  two insertions at the same `at` keep push order (stable sort).
- `locate` already resolves nested tables with
  `table(range, depth, key)` (checks the value is `{` and uses `closes[start]`
  for the end), so "does `Items[0]` have a `Settings` table" is
  `field(text, &lines, item, 4, "Settings")` with the value `{`, at the item's
  depth; `Sidecar.Version` is at depth 1 and the `Version` inside `Settings`
  at depth 5, so neither can collide.
- Tests in `dop.rs` pin: the exact fresh template text
  (`the_fresh_template_is_the_documented_one`), byte-for-byte patching of the
  PhotoLab fixtures `crates/core/src/fixtures/dop/_DSC0001..0005` (10.0.1,
  unindented, trailing CRLF) and `_DSC0009..0015` (10.0.2, tab-indented), and
  insertion of missing `Rating` / `ShouldProcess` / `ColorLabel` lines. All
  fixtures already have a full `Settings` block.
- `crates/app/src/sidecar.rs` calls `dop::write_rating` / `dop::write_label`
  (lines ~119, ~140, ~617-625) and needs no change.
- `README.md` / `README.ja.md` describe only which file gets written and that
  foreign sidecars are edited in place; they do not describe the template's
  keys. `CLAUDE.md` does not either.

## Steps

- [x] Step 1: Add the minimal `Settings` block to the fresh `.dop` template and insert it when patching a Settings-less item
  - Done when:
    - `template` emits `Settings = {\nVersion = "21.0",\n}\n,\n` in
      `Items[0]` between the `Rating` and `ShouldProcess` lines, and nothing
      else new (no preset, no `Orientation`, no `Base` / `Overrides`).
    - `write_rating` and `write_label` with existing bytes whose `Items[0]`
      has no `Settings` key (depth 4) insert the same block once as part of
      that write, **immediately before the `ShouldProcess` line** (the
      placement verified in PhotoLab; decided with the user), or before the
      item's closing brace when there is no `ShouldProcess` line (in that case
      ordered before an inserted `ShouldProcess` line); patching the result
      again does not duplicate it; the existing PhotoLab fixtures (all of
      which have a `Settings` block) patch byte-for-byte as before except the
      current splices, so every existing fixture-equality test stays green
      unchanged.
    - `read_rating` / `read_flag` / `read_label` round-trip on the new
      template and on a repaired old template.
    - Tests updated / added: the pinned template text; a fresh template
      round-trip (already exists, must keep passing); a test that builds
      Riffle's old template (the new template with the `Settings` lines
      removed), patches it with `write_rating` and with `write_label`, and
      checks the block appears exactly once, at the expected position, and
      that a second patch is idempotent; a test for an old template with no
      `ShouldProcess` line; a test that an existing `Settings` block
      (unindented `_DSC0003` and tab-indented `_DSC0009`) is not touched.
    - The module doc comment in `dop.rs` mentions that PhotoLab 10 refuses an
      item without a `Settings` table and that the writer adds the minimal
      one.
    - `learnings.md` of this plan records the bisection result and that the
      archived Step 5 claim was wrong; the archived learnings' Step 5 bullet
      gets a one-line "Correction (2026-09-26)" note pointing to this plan.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep the block in `template` itself (one function). The `Version`
      value is the same `"21.0"` as `Sidecar.Version`; a shared constant is
      fine but not required.
    - In `locate`, record whether `Items[0]` has a `Settings` key at depth 4
      (present whenever the key line exists at that depth, so nothing is ever
      inserted twice), and the start of the `ShouldProcess` line when present.
    - In both `write_rating` and `write_label` (existing-bytes path), when
      `Settings` is absent, push one more edit inserting
      `{indent}Settings = {\n{indent}Version = "21.0",\n{indent}}\n{indent},\n`
      (or the unindented form; Riffle's own files are unindented) at the start
      of the `ShouldProcess` line, falling back to `doc.item_close`. Keep the
      push order deliberate so the bytes are alphabetical when both
      `Settings` and `ShouldProcess` are inserted at the same point, and
      assert the exact resulting bytes in tests.
    - Do not add a bulk-migration command or any change in
      `crates/app/src/sidecar.rs`; the repair rides on the next judgment
      write.
    - Docs: `README.md` / `README.ja.md` need no change unless the implementer
      decides to note that "PhotoLab needs a `Settings` table, Riffle writes
      a minimal one"; if `README.md` changes, change `README.ja.md` in the same
      PR. `CLAUDE.md` needs no change.

## Trade-offs and risks

- Placement of the inserted block in patched files: decided with the user to
  insert it immediately before `ShouldProcess`, the layout verified in
  PhotoLab, rather than before the item's closing brace (simpler but
  unverified).
- The repair fires on any write to a Settings-less `.dop` (rating, flag or
  label), so PhotoLab's `ModificationDate` / `Date` also move, as they already
  do for every write; no extra risk.
- A `Settings` key present but not a table (`Settings = 1,`) is not a shape
  PhotoLab or Riffle produce; treating any depth-4 `Settings` line as present
  avoids ever inserting a second key. No handling beyond that.
- After the merge, the user should confirm one repaired file in PhotoLab
  (a folder PhotoLab has not indexed yet, since its database can shadow the
  sidecar).

## Progress

- (2026-09-26) Step 1 complete
