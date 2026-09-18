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

# DxO PhotoLab `.dop` sidecars, selected by a sidecar format setting

## Purpose

Phase 6 records a judgement (`1`-`5`, `x`, `u`, `0`) in an XMP sidecar. The
user's downstream tool is DxO PhotoLab, which keeps its own sidecar,
`<name>.ARW.dop`, and does not read `xmp:Rating="-1"` as a reject. Writing an
XMP next to every RAW is noise for a user who only uses PhotoLab. This work
adds `.dop` as a second sidecar format, behind a setting that selects **one**
format at a time: Riffle reads and writes only the selected format, and
ratings and rejects set in PhotoLab are read back from `.dop` when it is
selected. The default stays XMP, so existing behaviour is unchanged until the
user switches.

Decisions settled with the user; do not reopen:

1. Both directions when `.dop` is selected: Riffle's judgements are written
   to `.dop`, and PhotoLab's ratings/rejects are read from it.
2. When a file has no `.dop`, Riffle creates a minimal one PhotoLab accepts.
   What "minimal" PhotoLab tolerates is unknown at planning time; the user
   confirms it in PhotoLab (Step 2, **manual**), with a fallback described
   there.
3. One format at a time, selected by a setting. **No** writing of both
   sidecars and no mtime reconciliation between them. When the setting
   changes, the judgements are re-read from the newly selected format.

Samples: five real PhotoLab 10.0.1 sidecars the user made (0001 pick, 0002
reject, 0003 three stars, 0004 red colour label, 0005 three stars then back to
0). The relevant keys sit directly under `Sidecar.Source.Items[0]`:
`ShouldProcess` (0 = pick, 1 = reject, 2 = unflagged), `Rating = 0..5`, and
`ColorLabel = "Red"` (absent when there is no label). Line endings are LF with
a single CRLF on the last line.

Model mapping (Riffle has no colour label; pick is added in Step 4):

- Read: `ShouldProcess = 1` reads as a reject (`-1`), regardless of `Rating`;
  otherwise `Rating` `1`-`5` reads as stars and `Rating = 0` as `Some(0)`
  (explicit unrated, as `xmp::read_rating` does for `"0"`); a file with
  neither key reads as `None`. `ShouldProcess = 0` (pick) is not a reject and
  contributes nothing else.
- Write a reject: `ShouldProcess = 1`; `Rating` is left as it is.
- Write stars or clear: `Rating = n` (`0` for clear); `ShouldProcess` becomes
  `2` unless it currently is `0`, which is preserved (un-rejecting or rating
  a picked file must not drop the pick).
- `ColorLabel` and every other byte are preserved, except the patched keys
  and the two timestamps `Sidecar.Date` and `Items[0].ModificationDate`,
  which are set to the write time (see "Trade-offs and risks" for why).

## Steps

- [x] Step 1: Core: `.dop` read, patch and template (`crates/core/src/dop.rs`)
  - Done when:
    - `riffle_core::dop` exposes the same surface as `riffle_core::xmp`:
      `sidecar_path(&Path) -> PathBuf` (appends `.dop` to the **full** file
      name: `_DSC0001.ARW` -> `_DSC0001.ARW.dop`, never `set_extension`),
      `read_rating(&[u8]) -> Result<Option<i8>, String>`, and a
      `write_rating` that takes the existing bytes (or `None`), the rating
      and the write timestamp, and returns the new bytes.
    - Unit tests cover the five PhotoLab samples: 0001 (`ShouldProcess = 0`,
      `Rating = 0`) reads `Some(0)`; 0002 (`ShouldProcess = 1`) reads
      `Some(-1)`; 0003 (`Rating = 3`) reads `Some(3)`; 0004 (`ColorLabel =
      "Red"`, and eight decoy `Label = "..."` entries deeper inside
      `Settings`) reads `Some(0)` and keeps `ColorLabel` byte-for-byte after a
      patch; 0005 (rated then cleared, `Rating = 0`) reads `Some(0)`.
    - Patch tests assert byte identity outside the spliced values: writing
      `3` on 0001 changes exactly `Rating`, `ShouldProcess` (0 stays 0, the
      pick is kept), `Date` and `ModificationDate`; writing `-1` on 0003
      changes `ShouldProcess` to `1` and leaves `Rating = 3`; writing `None`
      on 0002 sets `Rating = 0`, `ShouldProcess = 2`; writing on 0004 keeps
      `ColorLabel` and the `Label` decoys; the trailing CRLF survives.
    - A key absent from the item (older PhotoLab, or a hand-made file) is
      inserted as its own `Key = value,` line at the item's nesting level;
      a test covers a file missing `ShouldProcess` and one missing `Rating`.
    - A fresh template round-trips through `read_rating` for `-1..=5`, and
      a test pins its exact text (like `the_fresh_template_is_the_documented_one`
      in `xmp.rs`).
    - Not parseable (no `Sidecar = {`, no `Items`, unbalanced braces,
      non-UTF-8) is `Err` for both read and write; an out-of-range `Rating`
      reads as `None`, mirroring `xmp.rs`.
    - `mise run ci` passes.
  - Implementation approach:
    - Mirror `crates/core/src/xmp.rs` in structure and doc style: locate the
      byte range of each value, splice, never regenerate an existing file.
      `pub mod dop;` in `crates/core/src/lib.rs`.
    - No Lua parser. A small scanner walks the text tracking brace depth,
      skipping over double-quoted strings (with backslash escapes, since a
      preset name or keyword could contain `{`, `}` or `"`), and remembers
      the byte range of the first table inside `Sidecar.Source.Items`. Within
      that range, at that depth only, it finds lines of the form
      `^\s*Key = value,` for `Rating`, `ShouldProcess`, `ModificationDate`,
      and at depth 1 `Date`. Depth tracking is what keeps the `Label = "Red"`
      entries inside `HSLHueSlices` from matching; do not use a flat search.
    - Insertion point for a missing key: before the item's closing `}`, on
      its own line, matching the file's convention (no indentation, `,`
      after the value, LF).
    - `write_rating` takes the timestamp as a parameter (a preformatted
      `&str`, or a `SystemTime` formatted inside) so the tests are
      deterministic. The format is PhotoLab's: `2026-09-18T10:20:50.0471837Z`
      (UTC, seven fractional digits). Formatting a `SystemTime` as a UTC
      civil date needs either a hand-rolled days-to-civil conversion (about
      20 lines, with a test against a known instant) or the `time` crate;
      pick one in this step and note it in `learnings.md`.
    - The fresh template is the **minimal** shape: `Sidecar = {` `Date`,
      `Software = "riffle"`, `Source = { Items = { { CreationDate,
      ModificationDate, Name = "<file name>", Rating, ShouldProcess, Uuid } },
      Uuid }`, `Version = "21.0"` `}`, LF line endings, no `Settings` block,
      no `CafId`, no `ShotDate`. Whether PhotoLab accepts it is checked in
      Step 2; keep the template in one function so the fallback there is a
      local change.
    - The two `Uuid`s need 128 random bits. Either add the `uuid` crate
      (`v4` feature) to `crates/core`, or generate them from
      `std::collections::hash_map::RandomState` hashers; decide in this step
      (the `uuid` crate is the conventional choice and the dependency is
      small).
    - Fixtures: copy the samples from
      `/Users/mino/Downloads/sony photolab flags/`: `_DSC0004.ARW.dop` **in
      full** (10 KB; it is the one with `ColorLabel` and the `Label` decoys)
      and trimmed copies of the other four (header, the whole `Items[0]` key
      set, a `Settings` block shortened to a few keys plus one nested table
      containing a `Label = "..."` decoy, the closing lines, and the final
      CRLF) under `crates/core/src/fixtures/dop/` via `include_str!` /
      `include_bytes!`, or inline `concat!` strings as `xmp.rs` does for the
      trimmed ones. Strip nothing else from the full copy; the UUIDs and
      dates in it are not sensitive.

- [x] Step 2: App: write and reconcile the selected format
  - Done when:
    - A `SidecarFormat { Xmp, Dop }` enum in `crates/app/src/sidecar.rs`
      dispatches `sidecar_path`, `read_rating`, `write_rating` and the file
      name match used by the folder listing (`.xmp` by extension; `.dop` by
      the `.arw.dop` suffix, both case-insensitive).
    - Settings move to `tauri-plugin-store` (user decision, 2026-09-18: more
      settings are coming, including a configurable keymap later). One store
      file (e.g. `settings.json` in the app config dir) holds
      `sidecarFormat` (`"xmp"` or `"dop"`; missing or unknown means `Xmp`)
      and `lastFolder`. The existing `last_folder` plain file is migrated
      once: if the store has no `lastFolder` and the old file exists, its
      value is copied in and the old file removed; existing `last_folder`
      tests are ported. The plugin is registered in `main.rs`, and the
      capability grants only what the app uses (the frontend does not need
      store access in this step). The format is read at startup from the
      store, held in managed state, and used by `sidecar::write` and by
      `reconcile_sidecars_of`. There is no UI yet; the store file is edited
      by hand to test. A format is carried with each `Writer::set` / `set_now` (in
      `Message::Set` and the `Pending` map) rather than read by the thread,
      so a judgement is written in the format selected when it was made.
    - Every existing sidecar test in `sidecar.rs` and `commands.rs` still
      passes for XMP, and each gets a `.dop` counterpart where the behaviour
      is format-specific: patch-in-place of a PhotoLab sidecar (the Step 1
      fixture), creation of the minimal template on a file with none,
      "clearing on a file with no sidecar writes nothing" (same rule as XMP),
      reading a PhotoLab-made reject and rating on the first open, a
      `.DOP`/`.ARW.DOP` differing only in case being the one that is read and
      patched, and the oversize (`MAX_SIDECAR_BYTES`) rule.
    - The PhotoLab manual check moved to Step 5 (user decision, 2026-09-18:
      check everything once, after pick lands).
    - `mise run ci` passes.
  - Implementation approach:
    - Keep `list_sidecars_in` as one directory listing; it takes the format
      and filters by it. The lower-cased-name map already handles case.
    - `reconcile_sidecars_of` and `sidecar::write` call the enum's dispatch;
      the six reconciliation rules and the dirty-snapshot pattern
      (`docs/agents/tauri-app.md`, "Snapshot the state you decided on") are
      unchanged. Column names `xmp_size` / `xmp_mtime_ns` stay: they now mean
      "the selected format's sidecar", say so in the `ratings` table comment
      in `index.rs`; renaming would need a `SCHEMA_VERSION` bump that drops
      dirty rows.
    - Reading the preference: through the store (`StoreExt::store`), replacing
      the plain-file helpers around `last_folder_file` in `commands.rs`; a
      read failure logs and falls back to `Xmp`. The state type lives with `AppWriter` /
      `AppIndex` (`pub struct AppSidecarFormat(Mutex<SidecarFormat>)` or an
      atomic).
    - Do not touch the frontend in this step.

- [x] Step 3: App: the setting itself (menu, command, index reset, reopen)
  - Done when:
    - A native `Sidecar` submenu exists in **all** builds with two check
      items, `XMP (.xmp)` and `DxO PhotoLab (.dop)`, exactly one checked,
      reflecting the persisted value at launch. Choosing the other item
      switches the format.
    - Switching: drains the writer (`flush(DRAIN_TIMEOUT)`), persists the new
      value to the store's `sidecarFormat`, updates the managed state, resets the index
      (`DELETE FROM ratings WHERE dirty = 0`; `UPDATE ratings SET xmp_size =
      NULL, xmp_mtime_ns = NULL WHERE dirty = 1`), then emits a
      `sidecar-format` event with the new value. Dirty rows survive on
      purpose: a judgement that never reached the old format is written into
      the new one on the next open (rule 4), which is the user's most recent
      intent.
    - The frontend listens for `sidecar-format` and, when a folder is open,
      reopens it (`openDirectory(openDir, newFolderToken())`), so the strip
      badges and the meta pane show the newly selected format's judgements.
      With no folder open nothing happens.
    - An `Index` unit test covers the reset (clean rows gone, dirty rows kept
      with their stat cleared), and a `commands.rs` test covers "switch
      format, reopen: the other format's ratings are gone and the selected
      format's are read".
    - `mise run ci` passes.
  - Implementation approach:
    - Restructure `main.rs`'s menu building so `Menu::default(handle)` plus
      the `Sidecar` submenu is built unconditionally and the `Debug` submenu
      is appended only under the existing `cfg`. Follow `debug_menu`'s
      pattern: manage the `CheckMenuItem`s so the handler can set the checks
      (uncheck the other item; the platform toggles only the clicked one).
    - The menu event handler runs on the main thread. The switch does IO
      (writer drain up to 2 s, SQLite, the config file), so the handler must
      hand it to `tauri::async_runtime::spawn_blocking`, per "Synchronous
      commands run on the main thread" in `docs/agents/tauri-app.md`. Emit
      the event from the blocking task once the reset is done, not before,
      so the frontend's reopen sees the reset rows.
    - The frontend listener is registered once and outlives folders; guard
      its work with the folder token as `docs/agents/tauri-app.md` requires
      ("Give the current folder one token").
    - Optional and small: show the selected format's sidecar name in the meta
      pane's sidecar section if that is where the user would look; do not
      add a settings dialog.

- [ ] Step 4: Pick flag (`.dop` only)
  - User decisions (2026-09-18): `p` picks the current file; a reject is
    replaced by the pick and vice versa (`x` on a picked file rejects it);
    `u` clears a pick as well as a reject; `0` clears the rating and, as
    today, the reject, and leaves a pick alone. Pick exists only while `.dop`
    is selected: XMP has no standard pick field, so with XMP selected `p`
    does nothing (no custom XMP property, no index-only pick).
  - Done when:
    - Riffle's judgement model carries the pick separately from the stars
      (pick and stars coexist, as `ShouldProcess = 0` and `Rating` do in
      `.dop`); the index stores it (schema change handled per the existing
      `SCHEMA_VERSION` rules, without dropping dirty rows silently).
    - `dop::read_*` reports `ShouldProcess = 0` as a pick and `dop` writing
      sets `ShouldProcess` 0/1/2 from pick/reject/neither; a Riffle write
      never clears a pick it did not mean to clear.
    - The frontend: `p` key, a pick badge on the strip cell and in the meta
      pane's sidecar section, and the README key table. With XMP selected,
      `p` is a no-op.
    - Unit tests: pick round trip on the fixtures (0001 reads as picked),
      pick replacing a reject and back, `u` clearing a pick, `0` keeping it,
      and XMP ignoring pick.
    - `mise run ci` passes.
    - The PhotoLab manual check for pick is in Step 5.

- [ ] Step 5: Documentation and the user's confirmations
  - Done when:
    - **(manual)** With `sidecarFormat` set to `dop` in the store: the user rates and
      rejects files in Riffle, then opens the folder in PhotoLab 10 and
      confirms (a) the folder opens without a sidecar error, (b) a rating and
      a reject set in Riffle on a file that already had a PhotoLab `.dop`
      show in PhotoLab, (c) a rating set in Riffle on a file that had **no**
      `.dop` (the minimal template) shows in PhotoLab, and (d) after PhotoLab
      changes a rating, reopening the folder in Riffle shows PhotoLab's
      value. If (c) fails, the fallback is a template that also carries the
      sample's `Settings` block (the `_DSC0004` fixture minus the per-file
      keys) as a constant; record which shape PhotoLab accepted in
      `learnings.md`. If (b) fails only for files PhotoLab already knew
      (PhotoLab preferring its database over the sidecar), that is a PhotoLab
      preference ("sidecar: load settings automatically") and is documented,
      not fixed here.
    - **(manual)** A pick set in Riffle shows as a pick in PhotoLab, and a
      pick set in PhotoLab shows in Riffle.
    - `README.md`: the "Ratings and XMP sidecars" section is renamed to cover
      both formats and documents the setting (menu, default XMP, one format
      at a time, what a switch does to the index and to unwritten
      judgements), the `.dop` mapping (`ShouldProcess` 0/1/2, `Rating`,
      pick preserved, `ColorLabel` untouched, the two timestamps updated),
      the minimal template and which shape PhotoLab accepted, and the
      "confirmed / verified without a GUI / awaiting confirmation" split
      records the manual results, including the PhotoLab
      database-versus-sidecar caveat if hit. The "Status" paragraph mentions
      `.dop`.
    - `CLAUDE.md` "Layout" mentions `crates/core/src/dop.rs` and the setting.
    - `docs/agents/tauri-app.md` gains any pitfall Steps 1-3 hit (a native
      menu in release builds, or the PhotoLab template, are candidates).
    - `mise run ci` passes.

## Trade-offs and risks

### Default format

Kept at XMP so nothing changes for an existing user until they switch, and
the app does not silently change which files it writes. The argument for
defaulting to `.dop` is that PhotoLab is the tool the user culls into. The
default is a one-line constant.

### Updating `Date` and `ModificationDate` on a patch (decided: update both)

PhotoLab keeps its own database and decides whether a sidecar is newer than
it. The file's mtime is one signal; the `Date`/`ModificationDate` inside are
the only others. Leaving them at PhotoLab's last write risks PhotoLab judging
its database newer and ignoring Riffle's change, which would make (b) in Step
2 fail for no visible reason. Updating them costs two more splices of the
same kind and cannot make PhotoLab ignore the file. Alternative: patch only
`Rating`/`ShouldProcess` (strictly minimal bytes). If the user's Step 2 test
shows the timestamps do not matter, keep updating them anyway; it is what
PhotoLab itself does on every write.

### A reject leaves `Rating` as it is

In XMP a reject *is* the rating (`-1`), so stars and a reject cannot coexist.
In `.dop` they are separate keys. Leaving `Rating` untouched on a reject
preserves PhotoLab's stars through a Riffle reject (a later un-reject in
PhotoLab gets them back); Riffle itself reads the file as `-1` either way.
Alternative: also write `Rating = 0` on a reject, for parity with the XMP
model. Not taken because it destroys information Riffle never displays. Note
the asymmetry: Riffle's `u` (un-reject) writes `Rating = 0`, because in
Riffle's model `u` means "no judgement"; a user who wants PhotoLab's stars
back should un-reject in PhotoLab.

### Index reset on a format switch versus a per-row format column

Taken: reset (`DELETE` clean rows, null the stat on dirty rows) and reopen.
Alternative: a `format` column on `ratings`, with reconciliation ignoring
rows of the other format, which would let a switch back restore the previous
cache without re-parsing. That needs a schema migration (`SCHEMA_VERSION` 2
to 3 with `ALTER TABLE`, or a bump that drops dirty rows) for a rare action
whose cost is one folder open. Not taken.

### Dirty rows on a switch

Taken: a dirty judgement is written into the **new** format on the next open.
Alternative: write it into the old format before switching (the drain does
that for what is already queued in the writer; a row that is dirty because
a previous write failed — read-only folder — would need the old format kept
around). The chosen rule is simpler and the judgement is not lost either way;
it just lands in the format the user just chose.

### The minimal `.dop` template

Unknown whether PhotoLab accepts a `.dop` with no `Settings` block. Risk:
PhotoLab may treat it as corrupt and either ignore it or refuse to open the
folder (worse). Mitigation: Step 2's manual check before anything ships,
with a documented fallback (embed the sample's `Settings` block). The
fallback ties the template to PhotoLab 10's `Version = "21.0"`; an older
PhotoLab may migrate or reject it, which is a limit the README should state.

### Parsing by depth-tracked line scan rather than a Lua parser

Enough for files PhotoLab writes (one key per line, no comments, no
multi-line strings). A hand-edited `.dop` with `--` comments or long-bracket
strings would be misparsed; both read and write return `Err` on anything the
scanner cannot balance, and an unparseable file is never overwritten, same
as the XMP rule.

### Menu in release builds

Today the app sets no custom menu in a distributable build, so it gets the
platform default. Setting one to add `Sidecar` replaces the default set;
`Menu::default(handle)` first (as `debug_menu::build` does) keeps the
standard items. The one change visible to a user who never switches is a new
menu; a keyboard shortcut or a settings dialog were rejected as larger.

### Manual verification

GUI automation is impossible on this machine, so everything involving
PhotoLab and the running app is **(manual)** for the user. Unit tests cover
the byte-level behaviour on the five real samples; they cannot tell whether
PhotoLab agrees.

## Progress

- (2026-09-18) Step 1 complete
- (2026-09-18) Step 2 complete (PhotoLab manual check deferred to Step 5)
- (2026-09-18) Step 3 complete
