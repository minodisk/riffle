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

# Strip multi-selection and bulk judgements

## Purpose

The strip (`crates/app/ui/src/strip.ts`) has exactly one selected file, the
current one, and every judgement key (`judge` in `crates/app/ui/src/main.ts`)
acts on `files[index]` alone. Marking ten frames of a shoot with the same
label or rating means ten key presses on ten files.

After this work the user can build a multi-selection in the strip with
Cmd/Ctrl+click (toggle one), Shift+click (range from the anchor) and
Shift+ArrowUp/Down (extend the range from the keyboard), and every judgement
that works on one file today (stars, reject, pick, un-flag, clear, the seven
colour labels and clear-label) applies to every selected file as one undoable
batch. A plain click or a plain arrow key collapses back to the single
selection the app has today, so single-file culling is unchanged. Sidecars
and the SQLite index are updated per file through the existing `set_rating`
command; no backend change is needed.

## Decisions taken at planning time

- The selection is keyed by path (a `Set<string>` plus an anchor path), like
  `ratings` and `picks`, because `refilter` rebuilds `files` and its indices.
- The focused file (`files[index]`, the one shown in the viewer) is always a
  member of the selection. When the selection has more than one member the
  strip shows the focused file with the existing `.current` style and the
  other members with a new, weaker `.selected` style.
- **Same value on every file (user requirement).** A judgement with a
  multi-selection computes the new value once, from the **focused file's**
  state, exactly as `judge` does today, and sets that same value on every
  selected file (a label key on a mixed selection sets the colour on all when
  the focused file lacks it, and clears it on all when the focused file has
  it; `x` rejects all; `0` clears all). Files whose state already equals the
  new value are skipped, so the batch holds real changes only, as
  `rejectRest` does.
- **Unrelated fields are preserved per file (user requirement).** A command
  changes only the field(s) it is about; every other field keeps each file's
  own existing value. A star key does not touch the colour label (and a label
  key does not touch the stars or the pick / reject flag) of any selected
  file. The per-file `Judgement` written for each target is that file's own
  `before` with only the command's field replaced, never the focused file's
  whole judgement copied over.
- Bulk judgements apply to the selected files that are **visible** (in
  `files`). A file the filter hides is dropped from the selection when the
  filter, sort or a judgement hides it; it is never judged invisibly.
- Auto-advance does not fire after a judgement applied to more than one file;
  the focused file stays where it is (a judgement that hides it under the
  filter still hands the cursor on, as today).
- Plain ArrowUp/Down, ArrowLeft/Right, a plain click and a folder open
  collapse the selection to the new focused file. Shift+ArrowUp/Down are new
  rebindable actions (`extendPrevious`, `extendNext`) in the keymap.
- Escape does not collapse the selection (not planned).

## Steps

- [x] Step 1: Pure selection model with tests
  - Done when:
    - `crates/app/ui/src/selection.ts` (new) exports a DOM-free model of the
      selection: the state (`selected: Set<string>`, `anchor: string |
      undefined`) and functions for a click on `files[at]` with `{ toggle,
      range }` modifier flags (plain: collapse to that file and set the
      anchor; toggle: add or remove it, never removing the focused file
      itself, and set the anchor; range: select the files between the anchor
      and `at` inclusive, keeping the anchor), for a keyboard extend by
      `delta` from the focused index (range from the anchor to the new
      focused index), for collapsing to one file, and for pruning to a list
      of paths (a filter or a trash that removed files) with the anchor
      falling back to the focused file when it is pruned
    - The model also exposes the "targets" of a judgement: the selected paths
      in `files` order when the selection has more than one member, else the
      focused file alone
    - `crates/app/ui/src/selection.test.ts` covers: plain click collapses;
      toggle adds and removes; toggle cannot remove the focused file; range
      after a toggle uses the toggled file as anchor; range replaces a
      previous range from the same anchor rather than accumulating; range
      with no anchor behaves as a plain click; keyboard extend grows and
      shrinks a range through the anchor; prune drops hidden paths and resets
      the anchor; targets order and the single-file fallback
    - `mise run ci` passes
  - Implementation approach:
    - Model it on `crates/app/ui/src/burst.ts` and `trash.ts`: exported
      functions over plain data, no DOM, no Tauri, so vitest's `node`
      environment runs it
    - Work in paths, not indices, at the state level; take `files: readonly
      string[]` as an argument wherever an index-to-path conversion is needed
    - Keep it small; do not add a class hierarchy or events

- [x] Step 2: Strip click modifiers and selection painting
  - Done when:
    - A Cmd/Ctrl+click on a strip cell toggles it in the selection and a
      Shift+click selects the range from the anchor; a plain click behaves
      exactly as today (selects and shows that file)
    - The focused file keeps the `.current` style; the other selected cells
      get a `.selected` style that is visibly weaker so the focused file
      stays distinguishable
    - Painting survives virtualisation: a selected cell scrolled out and back
      in is painted selected (`createCell` and `highlight` both consult the
      selection), and `setFiles` resets it
    - The status line under the strip shows the count when more than one
      file is selected (for example `12 / 80 · 3 selected`), following the
      `setStatus` pattern used for the burst position
    - The selection is cleared on folder open (`openDirectory`), pruned on
      every `refilter` (filter, sort, judgement), on `trashRejected` and on
      `resync`, using the Step 1 model
    - A Cmd/Ctrl+click on the focused file does nothing, and a click that
      only changes the selection does not reload the viewer
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Step 1 is merged
    - `strip.init(onSelect)` grows to pass the modifiers: hand the callback
      `(index, { toggle: event.metaKey || event.ctrlKey, range: event.shiftKey })`
      from the cell's click listener; keep `select` as the single callback
    - Add `strip.setSelected(indices: Iterable<number>)` mirroring
      `setRating`'s per-index Set pattern (`picks`), and extend `highlight`
      to toggle `.selected` on `index !== current && selected.has(index)`
    - In `main.ts`, keep `index` as the focused file; hold the Step 1 state
      next to `ratings`/`picks`; after every change to `files` or the
      selection, push the selected indices to the strip
    - Add the CSS rule next to `.cell.current` in `crates/app/ui/style.css`

- [x] Step 3: Bulk judgements over the selection
  - Done when:
    - Every action routed through `judge` in the keydown handler (`rate1`-
      `rate5`, `reject`, `pick`, `unflag`, `clear`, the seven labels,
      `clearlabel`) applies to all Step 1 "targets": one batch, one
      `history` entry, one `refilter`, one `set_rating` invoke per changed
      file, so the index rows and the sidecars (XMP or `.dop`) of every
      changed file are updated
    - The new value is computed once from the focused file's judgement and
      the same value is set on every target; targets already at that value
      are skipped; a batch with no change is a no-op that returns `false`
    - Only the command's field changes on each target; every other field
      keeps that file's own previous value (a star key leaves each file's
      label and flag as they were; a label key leaves each file's stars and
      flag as they were)
    - `Edit > Undo` restores all files of the batch in one step and
      `Edit > Redo` re-applies them (the existing `step` batch path)
    - Auto-advance does not move when the batch touched more than one file
    - A single selection behaves exactly as before
    - A failed `set_rating` for one file reverts that file only and leaves
      the rest applied
    - `rejectRest` is unchanged
    - A pure helper that turns `(targets, lookup, command)` into the
      `Change[]` batch lives in `selection.ts` (or a sibling module) and is
      covered by tests: mixed-state label toggle follows the focused file;
      the same value lands on all; unrelated fields of each file are
      preserved (e.g. a star command on files with different labels keeps
      each label); already-equal files are skipped; empty batch; the focused
      file is first so `commit`'s default anchor stays the focused file
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged
    - Express each command as "which field, which new value" computed from
      the focused file, then apply it to each target's own `before` so the
      other fields survive; do not copy the focused file's whole `Judgement`
    - Rewrite `judge` to build `changes` the way `rejectRest` does and call
      the same `commit(changes, forgetOnFail(history, batch), () => current)`
      with the focused file as anchor; keep the single-file case going
      through the same code rather than a special branch
    - The `pick` action keeps its `effectivePick` gate before any batch is
      built
    - Nothing in `crates/app/src/sidecar.rs` or `index.rs` should need to
      change; record any slowness on large selections in `learnings.md`

- [x] Step 4: Keyboard range extension and collapse
  - Done when:
    - `crates/app/src/shortcuts.rs` `DEFAULTS` gains `extendPrevious`
      (`shift+arrowup`) and `extendNext` (`shift+arrowdown`) after
      `burstNext`, with the Rust default/override tests extended to cover
      them
    - `crates/app/ui/src/settings.ts` `shortcutLabels` names them
      (`Extend selection up` / `Extend selection down`)
    - In `main.ts`, `extendPrevious`/`extendNext` move the focused file by
      one (showing it, as `move` does) and grow or shrink the selection from
      the anchor with the Step 1 extend function; `previous`, `next`,
      `burstPrevious`, `burstNext` and `move` collapse the selection to the
      new focused file
    - `mise run ci` passes
  - Implementation approach:
    - Assumes Steps 1-3 are merged
    - Follow `docs/agents/tauri-app.md` "An accelerator string is not
      validated until Tauri parses it": these actions have no menu item;
      confirm `accelerator()` returns `None` for them without erroring
    - Check the menus for a collision with `shift+arrowup`/`shift+arrowdown`

- [ ] Step 5: Documentation
  - Done when:
    - `README.md` Features (Filmstrip) and Keys mention Cmd/Ctrl+click,
      Shift+click, Shift+ArrowUp/Down and that judgements apply to the whole
      selection
    - `docs/usage.md` Features (Filmstrip, Judgements, Undo, Auto-advance)
      and Keys describe the selection rules above, including that the same
      value (taken from the focused file) is set on every selected file, that
      unrelated fields keep each file's own value, that hidden files leave
      the selection, and that auto-advance does not fire on a multi-file batch
    - `mise run ci` passes (lychee link check included)

## Trade-offs and risks

- **Hidden selected files.** Chosen: dropped from the selection and not
  judged, unlike `rejectRest`, so a filter change never marks files the user
  cannot see.
- **Auto-advance on a batch.** Chosen: no advance when more than one file
  changed; a later change can add it.
- **Per-file `set_rating` invokes.** N invokes for N files matches
  `rejectRest`; a batch command is deliberately not planned; measure first.
- **Ctrl+click on macOS** may become a `contextmenu`; then only Cmd+click
  toggles there, which matches platform expectations. Note it in
  `learnings.md` if observed.

## Progress

- (2026-09-22) Step 1 complete
- (2026-09-22) Step 2 complete
- (2026-09-22) Step 3 complete
