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

# Clear every flag of the current photo with one key

## Purpose

A photo can carry up to three independent judgements: stars or a reject
(`ratings`, `1`-`5` / `-1`), a pick (`picks`, `.dop` only) and a colour
label (`labels`). Clearing all of them today takes up to three keys (`0`,
`u`, `Ctrl+Alt+0`), and the user asked for one action that removes every
flag on the photo from its sidecar.

The backend already treats the three as one judgement: `set_rating`
(`crates/app/src/commands.rs`) takes `rating`, `pick` and `label` together,
`Index::set_rating` (`crates/app/src/index.rs`) stores them in one row, and
`sidecar::write` (`crates/app/src/sidecar.rs`) writes all three into one XMP
or `.dop` sidecar in one atomic rename. Every judgement key in
`crates/app/ui/src/main.ts` goes through `judge(next)`, which records one
undo batch. So this feature is a new keymap action, `clearall`, whose `judge`
callback returns `[null, false, null]`, plus the tests that pin down the
sidecar bytes such a judgement produces. No new Tauri command, no writer
change and no schema change are needed.

Riffle has no multi-select, so the action targets the current photo only,
like every other judgement key.

User decisions (2026-09-22):

- Default key: `c`.
- The existing `clear` action only clears the stars / reject. Its displayed
  name becomes "0 star" (the action id `clear` stays, so saved overrides keep
  working), and the new `clearall` action is displayed as "Clear".
- `clearall` always strips the label from the sidecar, even before
  `folder_entries` has told the frontend what label the file has: it sends
  `labelKnown: true` unconditionally.
- No confirmation dialog; the action is undoable like every judgement.

## Steps

- [x] Step 1: Pin down the "clear everything" judgement in the sidecar and index tests
  - Done when:
    - `crates/app/src/sidecar.rs` has tests, for both `SidecarFormat::Xmp`
      and `SidecarFormat::Dop`, that start from an existing sidecar holding
      stars plus a label (and, for `.dop`, a pick plus a label, and a reject
      plus a label), send one judgement of `rating: None, pick: false,
      label: None, label_known: true` through `Writer`, and assert the
      written file reads back as `read_rating == None`, `read_pick == false`
      and `read_label == None` (`.dop` also neither picked nor rejected). The
      XMP case asserts the `xmp:Rating="0"` convention documented by
      `clearing_writes_zero_into_an_existing_sidecar` in `crates/core/src/xmp.rs`.
    - A test asserts that the same judgement on a file with **no** sidecar
      writes nothing.
    - `crates/app/src/index.rs` has a test that `set_rating(dir, path, None,
      false, None, true)` on a row holding `Some(4), true, Some("Red")`
      leaves rating NULL, pick 0, label NULL, `label_known = 1`, `dirty = 1`.
    - If any of these fail, the fix lands in this step and is recorded in
      `learnings.md`.
    - `mise run ci` passes.
  - Implementation approach:
    - Extend the existing `judge(...)` test helper in `sidecar.rs` to take
      `pick: bool` rather than duplicating the `set_rating` + `writer.set` +
      `flush` sequence. Reuse the `PHOTOLAB_0003` fixture and
      `lightroom_sidecar()` helper.
    - Expected to be verification only.

- [x] Step 2: Add the `clearall` action: keymap default, key handler, settings labels
  - Done when:
    - `crates/app/src/shortcuts.rs` `DEFAULTS` has `("clearall", &["c"])`
      right after `("clearlabel", ...)`; `the_defaults_are_the_full_table`
      lists it and `the_defaults_bind_no_key_twice` still passes.
    - `crates/app/ui/src/main.ts` has a `case "clearall":` next to `"clear"`
      / `"clearlabel"` that calls `judge(() => [null, false, null])` and makes
      `send` pass `labelKnown: true` for this judgement even when the
      frontend does not yet know the file's label. It does not set `judged`,
      so Auto-advance does not fire.
    - `crates/app/ui/src/advance.test.ts` adds `"clearall"` to the "stays"
      list.
    - `crates/app/ui/src/settings.ts` `shortcutLabels`: `clear: "0 star"`,
      `clearall: "Clear"`.
    - Pressing `c` on a photo with stars/reject, a pick (`.dop`) and a label
      clears the strip cell, meta pane and filters immediately; undo restores
      all three at once; the sidecar reads back cleared in both formats.
    - `mise run ci` passes.
  - Implementation approach:
    - Use the existing `set_rating` path; no new Tauri command, `Writer`
      method or index method.
    - Force `labelKnown` by threading a flag from the `case` through `judge`
      → `commit` → `send`, not by special-casing the label comparison. Note
      it in `learnings.md`.
    - Confirm `c` is in no `forbidden()` / `*_MENU` / `*_SYSTEM` list in
      `shortcuts.rs`.

- [x] Step 3: Document the key
  - Done when:
    - `README.md` and `docs/usage.md` "Keys" tables have a row for `C`
      (clear every flag: stars, reject, pick and colour label) next to the
      `0` / `Ctrl+Alt+0` rows, and the `docs/usage.md` sentence about which
      keys leave the label alone mentions that `C` clears it too.
    - `mise run ci` passes (lychee link check included).

## Trade-offs and risks

- `labelKnown` forced to `true` for `clearall` deliberately departs from the
  "never strip a label we do not know about" invariant for this one action,
  because the user explicitly means "remove whatever is there".
- Scope is the current photo only; a burst-wide variant is out of scope.
- Sharpness is a computed score, not a flag, and is left alone.

## Progress

- (2026-09-22) Step 1 complete
- (2026-09-22) Step 2 complete
- (2026-09-22) Step 3 complete
