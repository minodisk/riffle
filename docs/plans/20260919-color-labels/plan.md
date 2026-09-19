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

# Colour labels

## Purpose

Riffle records stars, a reject and (with `.dop`) a pick, but has no notion of
a colour label, and the README promises `ColorLabel` is never touched. This
work adds the label as a third judgement field, read from and written to the
selected sidecar format so a label set in Lightroom or PhotoLab shows on the
strip, and a label set in Riffle shows in those tools. The label vocabulary
and the keys follow the format selected in the `Sidecar` menu:

- XMP (Lightroom): `xmp:Label` (attribute or element form, like
  `xmp:Rating`) holding the label name. Vocabulary `Red`, `Yellow`, `Green`,
  `Blue`, `Purple`; keys `6`, `7`, `8`, `9`, `-` (Lightroom's, with `-` for
  purple). Pressing the key of the label the file already has clears it.
- `.dop` (PhotoLab 10): the `ColorLabel = "Red",` line under
  `Sidecar.Source.Items[0]`; "no label" is the line being absent. Vocabulary
  `Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Pink`, `Purple`, verified from
  real PhotoLab 10.0.2.28 files (`_DSC0009`..`_DSC0015.ARW.dop` in
  `/Users/mino/Downloads/sony raw files`, one label each in that order).
  Keys match PhotoLab: `Ctrl+Alt+1`..`Ctrl+Alt+7` in that order and
  `Ctrl+Alt+0` clears; the same-key toggle applies too.

What the real files show (record in `learnings.md` when the fixtures land):

- The 10.0.2.28 files are **tab-indented** (one tab per depth) with LF line
  endings and `},` on one line; the existing 10.0.1 fixtures are
  unindented with `}\n,` and a trailing CRLF. `ColorLabel` sits
  alphabetically between `Albums` and `CreationDate`. The scanner's
  `field` uses `trim_start`, so tabs already parse, but nothing tests it
  and `Doc::edit` inserts an unindented line.
- The eight `Label = "Red"` ... `"Magenta"` lines deeper in every file are
  HSL hue slices, not labels; the depth-matched scanner keeps them apart.

Current state the plan is based on:

- `crates/core/src/xmp.rs` and `crates/core/src/dop.rs` each expose
  `read_rating` / `read_pick` / `write_rating`; a foreign sidecar is only
  ever patched by splicing byte ranges (`locate` in both files), never
  regenerated. `dop::write_rating` also splices the two timestamps and
  inserts a missing key as its own line before the closing brace.
- `crates/app/src/sidecar.rs`: `SidecarFormat` dispatches to the two
  modules; the writer thread carries `(rating, pick, format, deadline)` per
  path; `write` composes `existing_sidecar` -> `format.write_rating` ->
  temp file + rename, and skips a file with no sidecar when the judgement
  is empty.
- `crates/app/src/index.rs`: `SCHEMA_VERSION = 3`, `ratings (path, dir,
  rating, pick, xmp_size, xmp_mtime_ns, dirty)`; `set_rating`,
  `mark_written` (guarded on the row still holding the written value),
  `dirty_rows`, `reconcile_sidecars`, `store_sidecar_ratings`
  (`ParsedSidecar`), `reset_sidecars`, `entries` -> `IndexedFile`.
- `crates/app/src/commands.rs`: `reconcile_sidecars_of` parses rating and
  pick per changed sidecar; `scan_folder` hands dirty rows to
  `writer.set_now`; `set_rating(path, rating, pick)` is the keypress
  command; `switch_sidecar_format` applies a menu switch and `main.rs`
  emits `sidecar-format` afterwards; `load_settings` returns `(format,
  Keymap::from_overrides(store.get("shortcuts")))`.
- `crates/app/src/shortcuts.rs`: `DEFAULTS` (action -> keys), `Keymap`
  merges flat overrides, rejects `p` outside `pick` and any key already
  bound; key names are `event.key` lower-cased (`" "` -> `"space"`).
  `main.ts` builds `keymap` from the `shortcuts` command and its `keydown`
  handler returns early on `metaKey || ctrlKey || altKey`, then `switch`es
  on the action. The shortcuts plan's Steps 3-4 (rebind commands, panel)
  are not merged yet.
- Frontend judgement state: `ratings` map and `picks` set mirrored in
  `strip.ts`; `applyRating(path, rating, pick)`, `judge(next)`,
  `strip.setRating(index, rating, pick)`; the cell shows stars top-right
  (`span.rating`) and a pick/reject dot top-left (`span.flag`). The
  judgement is shown nowhere else (README lines about a `Rating` row in
  the meta pane are stale).

Design decisions taken by this plan (the user's, 2026-09-19, where marked):

- (user) Vocabulary and keys are linked to the selected format and switch
  with it. (user) Ctrl+Alt combinations become bindable.
- The label is stored and carried as the raw string the sidecar holds
  (`Option<String>`), not an enum. Reading returns the raw bytes of the
  value and writing splices them back as-is, so a name from the other
  vocabulary (Orange/Pink under XMP) or a localised Lightroom name
  round-trips untouched with no escaping code; the frontend colours every
  name it knows (all seven) and shows any other value grey. (user)
- Clearing removes the property (attribute, element or `.dop` line) rather
  than writing an empty value; an empty value is read as no label.
- The writer always writes the whole judgement (rating, pick, label) from
  the `ratings` row, as it already does for rating and pick, so a label
  keypress does not clobber stars and vice versa.
- Keymap model: one action per colour (`red`, `orange`, `yellow`, `green`,
  `blue`, `pink`, `purple`) plus `clearlabel`, format-independent names;
  only their **default keys** depend on the format (XMP: `6 7 8 9 -` on
  red/yellow/green/blue/purple, orange/pink/clearlabel unbound; `.dop`:
  `ctrl+alt+1`..`ctrl+alt+7`, `ctrl+alt+0` on `clearlabel`). User overrides
  stay one flat `shortcuts` object and apply under both formats; conflict
  checking runs against the current format's defaults. See Trade-offs.
- No filtering by label (the filter menu is untouched).

## Steps

- [x] Step 1: XMP: read and patch `xmp:Label`
  - Done when:
    - `xmp::read_label(bytes) -> Result<Option<String>, String>` returns the
      raw value of `xmp:Label` (attribute or element form, either prefix
      bound to the XMP namespace, first `rdf:Description` that has one),
      `None` when absent or empty; `Err` on the same inputs `read_rating`
      errors on.
    - `xmp::write_label(existing: Option<&[u8]>, label: Option<&str>) ->
      Result<Vec<u8>, String>`: with `Some(label)` splices the value in
      place, or inserts one attribute into the first `rdf:Description`
      (declaring the namespace when needed) exactly as `write_rating`
      does; with `None` removes the property — the attribute with the
      whitespace before it, or the whole element plus, when it sits alone on
      its line, that line — and is a byte-identical no-op when there is
      none. `existing` `None` with `Some(label)` is the fresh template with
      the attribute; with `None` it is an error (the caller never mints a
      sidecar for "no label").
    - `read_rating` / `write_rating` behaviour and every existing test are
      unchanged.
    - Unit tests: read attribute and element forms; absent and empty read
      as `None`; set on `BRIDGE` and `LIGHTROOM` changes only the value;
      insert into `NO_RATING`; clear removes the attribute and the element
      leaving everything else byte-identical; clear on a sidecar without
      one is a no-op; rating a labelled sidecar keeps the label and
      labelling a rated one keeps the rating (compose `write_rating` then
      `write_label`); a non-ASCII value and `"Orange"` round-trip
      byte-for-byte.
    - `mise run ci` passes.
  - Implementation approach:
    - Generalise `locate` to take the property's local name (`"Rating"` /
      `"Label"`); `Location::Value` gains the range to remove (attribute
      including leading whitespace, or start tag through end tag) while
      `write_rating` keeps splicing only the value.
    - Update the module doc ("Nothing but `xmp:Rating` is ever written").

- [x] Step 2: `.dop`: read and patch `ColorLabel`, with the PhotoLab 10.0.2 fixtures
  - Done when:
    - `_DSC0009.ARW.dop`..`_DSC0015.ARW.dop` are copied verbatim into
      `crates/core/src/fixtures/dop/` (about 13 KB each).
    - `dop::read_label(bytes) -> Result<Option<String>, String>` returns
      the string inside the quotes of `Items[0].ColorLabel`, `None` when
      the key is absent, empty, or not a double-quoted string.
    - `dop::write_label(existing: Option<&[u8]>, label: Option<&str>, name:
      &str, now: &str) -> Result<Vec<u8>, String>`: `Some(label)` splices
      the quoted value or inserts a `ColorLabel = "...",` line before the
      item's closing brace; `None` removes the whole line (line start
      through its newline) and is a no-op when absent. It updates
      `Sidecar.Date` and `Items[0].ModificationDate` like `write_rating`
      so PhotoLab sees the sidecar as newer. `existing` `None` with
      `Some(label)` is the template (`Rating = 0`, `ShouldProcess = 2`)
      plus the line; with `None` an error.
    - An inserted line copies the leading whitespace of the line holding
      the closing brace it is inserted before, so a tab-indented file gets
      a tab-indented line and the unindented fixtures stay unindented.
      `write_rating`'s own inserts (`Rating`, `ShouldProcess`, `Date`,
      `ModificationDate`) get the same treatment via the shared
      `Doc::edit`, and the existing insert tests still pass byte-for-byte
      (they insert into unindented files).
    - Unit tests: each of the seven new fixtures reads its label
      (`Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Pink`, `Purple`) and its
      rating/pick still read correctly (they are the first tab-indented
      inputs the scanner sees); the five old fixtures read `Some("Red")`
      for 0004 and `None` otherwise; setting `"Blue"` on 0004 and on 0009
      changes only that value and the two timestamps (`with_now`);
      inserting on 0003 (unindented) and on 0008-shaped input (tab-indented
      with no label: derive by removing the line from 0009) lands as its own
      correctly indented line before the item's closing brace and reads
      back; clearing 0004 and 0009 removes exactly the `ColorLabel` line
      and keeps all eight HSL `Label` lines, the trailing CRLF (0004) and
      the LF ending (0009); rating after labelling and labelling after
      rating keep each other; the existing
      `patching_keeps_the_colour_label_and_the_label_decoys` still passes.
    - `mise run ci` passes.
  - Implementation approach:
    - Add `color_label: Option<Range>` to `Doc` via `in_item("ColorLabel")`;
      the value range includes the quotes, so the read strips them and the
      write re-adds them.
    - Line removal is a small helper beside `Doc::edit`.
    - Note in `learnings.md`: `CafId` became `CafID` in 10.0.2 (irrelevant
      to the scanner, but worth knowing the key set is not stable).

- [x] Step 3: Index: store the label beside the rating
  - Done when:
    - `ratings` gains `label TEXT` (NULL = none). `set_rating`,
      `mark_written` (the guard also matches `label IS ?`), `dirty_rows`,
      `store_sidecar_ratings` / `ParsedSidecar`, `reconcile_sidecars`'s
      deleted-sidecar rule (clears `label` with `rating` and `pick`) and
      `entries` / `IndexedFile.label: Option<String>` all carry it.
      `reset_sidecars` keeps the label of dirty rows (both formats have
      one), unlike `pick`.
    - `SCHEMA_VERSION` is bumped; the previous version migrates in place
      with `ALTER TABLE ratings ADD COLUMN label TEXT` (dirty rows kept);
      older versions chain through the existing migrations; anything else
      still hits the `unsupported index schema version` discard.
    - Unit tests: a label round-trips through `set_rating` -> `entries`;
      `mark_written` with a different label leaves the row dirty; a
      previous-version fixture database with a `dirty = 1` row opens with
      the row intact and `label` NULL, and a reopen is the new version; the
      v2 migration test still passes; `reset_sidecars` keeps a dirty row's
      label.
    - Callers in `commands.rs` / `sidecar.rs` pass `None` for now (or this
      step merges with Step 4 if the intermediate is awkward). Behaviour
      unchanged. `mise run ci` passes.
  - Implementation approach:
    - Coordinate with the in-flight `20260919-exif-filters` plan, whose
      Step 2 also bumps `SCHEMA_VERSION` to 4. Whichever merges first takes
      v4; the other takes v5 and chains its migration after it so a v3
      database goes v3 -> v4 -> v5 in one open. Do not fold the two
      migrations together.
    - The `(rating, pick)` tuples grow to three fields in `dirty_rows`,
      `ParsedSidecar`, the writer's `Pending` and `Message::Set`. Either
      extend them or introduce one `Judgement { rating: Option<i8>, pick:
      bool, label: Option<String> }` in `index.rs`; pick whichever keeps
      the diff readable and use it consistently in Step 4.

- [ ] Step 4: Writer, reconcile and the `set_rating` command carry the label
  - Done when:
    - `SidecarFormat::read_label` / `write_label` dispatch to Steps 1-2.
      `sidecar::write` composes `write_rating` then `write_label` on the
      result (template case included); the "nothing to write" rule becomes
      `rating.is_none() && !pick && label.is_none()`. `Writer::set` /
      `set_now` / `mark_written` carry the label.
    - `reconcile_sidecars_of` parses the label with the rating and pick, so
      a label written by PhotoLab or Lightroom is stored on folder open;
      `scan_folder` hands dirty rows' labels to `set_now`.
    - `set_rating(path, rating, pick, label: Option<String>)` accepts any
      string or null (empty normalised to null) and stores it; XMP keeps it
      (unlike `pick`).
    - Tests in `sidecar.rs`: a label alone mints an XMP and a `.dop`; a
      label on a Lightroom-shaped sidecar keeps the `crs:` block and the
      rating byte-for-byte; a later rating keeps the label; clearing the
      label removes the property and leaves the rating; clearing a label on
      a file with no sidecar writes nothing. Tests in `commands.rs`: the
      seven new fixtures read their labels on first open with `.dop`, and
      an XMP with `xmp:Label="Blue"` reads with XMP; a deleted sidecar
      clears the label.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1-3 are merged. The label goes into `ParsedSidecar` and
      the `mark_written` guard ("snapshot the state you decided on",
      `docs/agents/tauri-app.md`).

- [ ] Step 5: Keymap: per-format defaults, the label actions, and Ctrl+Alt keys
  - Done when:
    - `shortcuts.rs`: `DEFAULTS` gains, after `clear`: `red`, `orange`,
      `yellow`, `green`, `blue`, `pink`, `purple`, `clearlabel`, each with a
      default per format — `Keymap::from_overrides(overrides, format)` (or
      `Keymap::for_format`) resolves the XMP defaults (`6`, none, `7`, `8`,
      `9`, none, `-`, none) or the `.dop` defaults (`ctrl+alt+1`..
      `ctrl+alt+7`, `ctrl+alt+0`). An action may have an empty default
      list; `parse_keys` keeps rejecting an empty override. The
      no-duplicate-defaults test runs per format.
    - Key names extend to `ctrl+alt+<key>` where `<key>` is derived from
      `event.code` (`Digit1` -> `1`, `KeyA` -> `a`, `Minus` -> `-`,
      `Space` -> `space`), because Option changes `event.key` on macOS.
      Plain names stay `event.key`-based, so existing bindings behave as
      before. Ctrl-only, Alt-only and any Meta combination stay ignored
      (left to the system). The reserved-`p` rule matches plain `p` only.
    - `main.ts`: `keyName(event)` returns the `ctrl+alt+...` form when
      `ctrlKey && altKey && !metaKey`, the handler no longer returns early
      for that combination, and calls `preventDefault` when the key is
      bound. Dispatch for the eight new actions is present but Step 6 wires
      the behaviour (until then the cases are a no-op, or Step 5 and 6
      merge as one PR if the reviewer prefers).
    - The overrides `Value` is kept in app state beside `AppKeymap` (e.g.
      `AppShortcutOverrides(Mutex<Option<Value>>)`), `load_settings` builds
      the keymap for the loaded format, and `switch_sidecar_format` rebuilds
      `AppKeymap` for the new format. The frontend re-invokes `shortcuts`
      in its `sidecar-format` listener and calls `applyKeymap`, so the keys
      follow the menu without a restart.
    - Unit tests: defaults per format; an override on `red` (`["r"]`)
      applies under both formats; an override that collides with one
      format's default is skipped under that format only (e.g. `reject:
      ["6"]` is skipped under XMP and accepted under `.dop`); `ctrl+alt+1`
      as an override on `reject` under XMP is accepted; `["p"]` and
      `["ctrl+alt+p"]` behave as documented (only plain `p` is reserved);
      the existing tests still pass with a format argument.
    - **(manual)** the user confirms on macOS: under `.dop`, ⌃⌥1 sets red
      and ⌃⌥0 clears (after Step 6); under XMP, ⌃⌥1 does nothing and `6`
      works; switching the menu swaps them without a restart.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes the shortcuts plan's Step 2 (merged). Rebase over its Steps
      3-4 if they land first: `rebind` / `overrides()` compare against the
      current format's defaults (see Trade-offs), and the panel's capture
      must use the same `keyName` so it can record `ctrl+alt+...`; the
      panel's TypeScript label table needs the eight new rows.
    - Represent the format-specific default as a pair of slices in
      `DEFAULTS` (`(action, xmp_keys, dop_keys)`) or as a `match` inside
      the resolver; keep the frontend free of defaults.

- [ ] Step 6: Frontend: show the label and set it from the keymap
  - Done when:
    - `IndexedFile.label: string | null` is declared; a `labels`
      `Map<string, string>` sits beside `ratings` / `picks` in `main.ts`
      and `strip.ts`; `applyRating`, `judge`, `refilter`, `refreshEntries`,
      the folder-open clears and `strip.setRating` carry it; `set_rating`
      is invoked with `label`.
    - The strip cell shows the label as a colour: seven CSS custom
      properties (`--label-red` ... `--label-purple`, matched
      case-insensitively on the name) next to `--stars-color`; any other
      non-empty value uses a neutral grey; `title` on the element carries
      the raw name. Where the colour goes (a tinted file-name strip like
      Lightroom's cell, or a thin bar along the cell's bottom edge) is
      decided in this step; it must not collide with the stars (top-right)
      or the dot (top-left).
    - `main.ts` dispatches the eight actions through `judge`, which becomes
      a function of `(rating, pick, label)`: a colour action sets that name
      (capitalised as the vocabulary spells it, `"Red"` etc.), or clears it
      when the file already has exactly that name (toggle); `clearlabel`
      clears; `clear` (`0`), `unflag`, `reject`, `pick` and the stars leave
      the label alone; the idempotence check compares the label too. The
      colour actions are not gated on the format: an action the user bound
      under XMP (say `orange`) writes `"Orange"` to the XMP, which
      Lightroom shows as a custom label.
    - **(manual, GUI automation is unavailable)** the user confirms, with
      `.dop` on the PhotoLab folder and with XMP on any folder: the seven
      fixtures show their colours on open; the keys set, replace and toggle;
      a rating after a label keeps it and PhotoLab / Lightroom show the
      label; clearing in Riffle clears it there; a label from the other
      vocabulary shows its colour.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 4-5 are merged. The existing `judge` callers pass the
      label through unchanged (`(rating, pick, label) => [stars, pick,
      label]`).

- [ ] Step 7: Documentation and the user's confirmations
  - Done when:
    - `README.md`: the key table gains the label keys per format and the
      Ctrl+Alt rule; "Ratings and sidecars" describes `xmp:Label` and
      `ColorLabel` (vocabularies, toggle, removal on clear, raw string kept
      for foreign names), the schema migration rule, and drops the stale
      "`ColorLabel` is never touched" / "no colour label" / meta-pane
      `Rating` row sentences; the manual results of Steps 5-6 go in the
      "confirmed / awaiting the user's confirmation" split. The
      `shortcuts` documentation (from the shortcuts plan, if merged)
      mentions the per-format defaults and the `ctrl+alt+` key names.
    - `CLAUDE.md` "Layout" mentions the label in the `xmp.rs` / `dop.rs`
      description and the per-format keymap in `shortcuts.rs`.
    - `docs/agents/tauri-app.md` gains any pitfall Steps 1-6 hit — at least
      the `event.key`-vs-`event.code` one if it bit.
    - `mise run ci` passes.

## Trade-offs and risks

### Per-format defaults with flat overrides (chosen) versus per-format overrides

- **Chosen**: action names are format-independent; only the defaults switch
  with the format; the `shortcuts` object stays flat and applies under
  both formats. One override edits one action everywhere, which is what a
  user who dislikes `-` for purple wants. Cost: the shortcuts plan's
  `overrides()` (Step 3, unmerged) becomes "differs from the *current*
  format's default", so a rebind that happens to equal the other format's
  default is stored, and a rebind back to this format's default is dropped
  even if the other format's default differs. Acceptable and documented.
- **Alternative**: `"shortcuts": {"xmp": {...}, "dop": {...}}`. Breaks the
  existing setting shape and doubles the panel. Not chosen.

### Toggle everywhere

The user asked for toggle under XMP; `.dop` also has an explicit clear key.
The plan makes the same-key toggle universal for consistency and keeps
`clearlabel` as well. If PhotoLab users find the toggle surprising the
`.dop` case is one conditional in `judge`.

### Ctrl+Alt on Windows and on non-US layouts

`Ctrl+Alt` is AltGr on many European Windows layouts, and `event.code`
names physical keys, so `ctrl+alt+1` is "the key in the `1` position", not
the character. Fine on macOS (the target), unverified on Windows; the plan
matches by `event.code` only for the modified form.

### Removing versus blanking on clear

Removing the property needs the XMP `locate` to report the removable range.
Chosen because absence is what both tools write for "no label" and an
empty attribute would be a Riffle-only shape; reading treats empty as none
so either survives.

### `.dop` insertion position and indentation

PhotoLab writes `ColorLabel` alphabetically; Riffle inserts before the
item's closing brace, as it already does for `Rating` / `ShouldProcess`
(accepted by PhotoLab 10.0.1). Indentation now follows the closing line.
Whether PhotoLab 10.0.2 re-reads a `.dop` for a label-only change (the
timestamps are updated as for ratings) is **awaiting the user's
confirmation** (Step 6).

### Schema-version collision with `20260919-exif-filters`

Both plans bump `SCHEMA_VERSION` from 3. The second to merge takes the next
number and chains its migration; check `prepare`'s accepted-version list on
rebase so two "v4" definitions cannot coexist.

### Localised or foreign label names

Round-trip byte-for-byte; coloured when the name is one of the seven, grey
otherwise. Mapping localised Lightroom names is out of scope.

### Manual verification

GUI automation is impossible on this machine (`docs/agents/tauri-app.md`),
so the strip colour, the keys (especially ⌃⌥ digits and the format switch)
and the PhotoLab / Lightroom round-trip are **(manual)**; unit tests cover
read/patch for both formats and both indentation styles, the migration,
the writer composition and the per-format keymap.

## Progress

- (2026-09-19) Step 1 complete
- (2026-09-19) Step 2 complete
