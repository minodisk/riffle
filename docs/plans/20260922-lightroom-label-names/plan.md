# Lightroom Classic colour label names

## Purpose

Lightroom Classic (verified on LrC 2026, Japanese UI, real files) decides a
photo's colour label only from the `xmp:Label` string, matched against the
names in the user's colour label set (Metadata > Color Label Set > Edit;
"Lightroom default" is Red/Yellow/Green/Blue/Purple in English and
レッド/イエロー/グリーン/ブルー/パープル in Japanese). It ignores
`photoshop:LabelColor` on read: an XMP with only `LabelColor="red"` gave no
label, one with only `xmp:Label="レッド"` gave red, and Riffle's current
`xmp:Label="Red"` + `LabelColor="red"` showed as white / unknown until the
user renamed their label set to English. LrC itself writes both
(`xmp:Label="レッド"` + `photoshop:LabelColor="red"`).

This answers the open question in `todo.md` ("App: the Lightroom 9.5.1
round-trip check is still open"): Lightroom does not resolve the colour from
`LabelColor`, so the label-write approach is revised. The user tells Riffle
the `xmp:Label` names their Lightroom uses: a per-colour name setting shown
only while the XMP ("Lightroom") format is selected, with a Japanese-default
preset and a reset to English. Writing uses the configured name for
`xmp:Label` (still `LabelColor` in lowercase English); reading keeps
`LabelColor` precedence and, when it is absent, maps a configured or English
name back to the colour, so files written with custom names round-trip. The
settings window's format choices are renamed "Lightroom" (XMP) and
"PhotoLab" (.dop) with the stored `sidecarFormat` values unchanged, and the
README gains a Lightroom Classic section on how LrC picks up sidecar changes.

Base: `origin/main` after #352 (`xmp.rs`, `sidecar.rs` and `settings.html`
are unchanged since the `lightroom-xmp-flags-labels` plan, now archived).

Decisions already taken (do not reopen):

- `xmp:Label` = the configured name for the colour; `photoshop:LabelColor`
  stays lowercase English. Default names are English (current behaviour).
- Reading: `LabelColor` first; when absent, match `xmp:Label` against the
  configured names and the English defaults; a name matching neither is
  still returned raw (today's behaviour, shows as grey "other").
- Persisted `sidecarFormat` values (`"xmp"` / `"dop"`) do not change; only
  the display names do.
- Orange and Pink are not in Lightroom's five and have no name field; they
  keep writing their English name as today.

Todo heading closed out by this plan (for the wrap-up's todo curation):
`### App: the Lightroom 9.5.1 round-trip check is still open`.

## Steps

- [x] Step 1: Label names in `crates/core/src/xmp.rs`
  - Done when:
    - `xmp.rs` (or `lib.rs`, next to `Flag`) has a `LabelNames` struct
      holding the `xmp:Label` string for each of Red, Yellow, Green, Blue,
      Purple (`Clone`, `Debug`, `PartialEq`, `Default` = English names), with
      a lookup by canonical colour name (`"Red"` ... `"Purple"`, the strings
      the app already passes) and a `japanese()` (or similarly named)
      constant for レッド/イエロー/グリーン/ブルー/パープル (the preset the
      settings UI offers; keeping it in core keeps the strings in one place).
    - `xmp::write_label(existing, label, names: &LabelNames)`: for one of the
      five canonical colours, `xmp:Label` is `names`' string for it and
      `LabelColor` is the lowercase English colour; for any other label
      (Orange, Pink, a foreign string) both are written as today.
    - `xmp::read_label(bytes, names: &LabelNames)`: `LabelColor` precedence
      unchanged; when `LabelColor` is absent or empty, an `xmp:Label` equal
      (trimmed) to a configured name or to an English default maps to the
      canonical colour (`"Red"` ...); otherwise the raw string as today.
    - Tests: a sidecar written with the Japanese names carries
      `xmp:Label="レッド"` + `LabelColor="red"` and reads back as `"Red"`
      both with `LabelColor` present and with it stripped; a sidecar holding
      only `xmp:Label="Red"` (English) reads as `"Red"` under Japanese
      names; `LIGHTROOM_LABELLED` with `LabelColor` stripped reads as
      `"Purple"` under Japanese names and raw `"パープル"` under English; a
      foreign name still reads raw; the existing byte-exact tests pass with
      `LabelNames::default()`; a `LabelNames` with an empty string for a
      colour falls back to the English default (test it; the app's Step 2
      validation relies on it).
    - `cargo test -p riffle-core` and `mise run ci` pass.
  - Implementation approach:
    - Keep the two-pass splice (`xmp:Label`, then `LabelColor`) and the
      removal path unchanged; only the value strings change.
    - Match names exactly after trimming (LrC compares its own strings; no
      case folding for non-ASCII).
    - Update the module doc comment (lines 7-13) to describe the configured
      name.
    - `crates/cli` does not call `read_label` / `write_label` (grep before
      changing the signature; fix call sites if it does).

- [x] Step 2: `labelNames` setting through the app: state, persistence, commands, parse and writer
  - Done when:
    - A `labelNames` key in the settings store persists the five names as a
      JSON object keyed by lowercase colour (`{"red": "レッド", ...}`);
      missing or non-string / empty entries fall back to the English default
      (a helper like `auto_advance_setting` in `commands.rs`, unit-tested).
    - `load_settings` returns the names; `main.rs` manages an
      `AppLabelNames(Mutex<LabelNames>)` state next to `AppSidecarFormat`.
    - Commands `label_names` (returns the five names in the same JSON shape,
      plus the Japanese preset if Step 3 chooses to fetch it) and
      `set_label_names(names)` (normalises, stores in state, persists under
      `labelNames`, and makes the open folder re-read its XMP sidecars; see
      Trade-offs) are registered in `main.rs`' handler list.
    - `SidecarFormat::read_label` / `write_label` take the names and pass
      them to `xmp::` (ignored for `Dop`); the sidecar parse in
      `commands.rs` (`format.read_label(&bytes)`, ~line 283) and the
      writer's `write()` in `sidecar.rs` use the names.
    - The writer receives the names the way it receives `format`: a snapshot
      carried in `Writer::set` / `set_now` / `Message::Set`, so a change
      after a judgement was queued does not rewrite it with a different
      name.
    - Tests in `sidecar.rs` / `commands.rs`: an XMP write under Japanese
      names yields `xmp:Label="レッド"`; a folder holding a sidecar with
      only `xmp:Label="レッド"` (no `LabelColor`) opens with label `"Red"`
      under Japanese names and raw `"レッド"` under English names.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - If `set_label_names` resets the index it touches SQLite, so make it
      `async` + `spawn_blocking` as `main.rs::set_sidecar_format` does (see
      `docs/agents/tauri-app.md`, "Synchronous commands run on the main
      thread"); a persist-only variant can stay sync like `set_auto_advance`.
    - Emit a `label-names` event carrying the stored names so the settings
      window refreshes its fields, mirroring `auto-advance`.
    - `.dop` is unaffected: `dop::read_label` / `write_label` keep their
      signatures.

- [ ] Step 3: Settings window: "Lightroom" / "PhotoLab" names and the label name fields
  - Done when:
    - `crates/app/ui/settings.html` shows the radios as "Lightroom (.xmp)"
      and "PhotoLab (.dop)" (values `xmp` / `dop` unchanged).
    - Under the Sidecar tab, a block visible only while the `xmp` radio is
      checked (hidden on `dop`; toggled in `showSidecarFormat` and the radio
      change handler in `settings.ts`) has one text input per colour (Red,
      Yellow, Green, Blue, Purple) labelled as the `xmp:Label` written for
      that colour, a short note that they must match the names in
      Lightroom's colour label set, a "Lightroom default (Japanese)" preset
      button that fills レッド/イエロー/グリーン/ブルー/パープル, and a
      "Reset to English" button.
    - Values load from `label_names`, save through `set_label_names` on
      `change` (per field, and after either button), and follow the
      `label-names` event; an error goes to `#status` like the other
      controls.
    - Any pure helper (building the payload, applying a preset) lives in a
      small module with a `*.test.ts` next to it, following `advance.ts` /
      `tabs.ts`; the DOM wiring stays in `settings.ts`.
    - `mise run ci` passes; the visibility rule is checked by hand in the
      app (switching the radio hides / shows the block without reopening the
      window).
  - Implementation approach:
    - Assumes Step 2 is merged.
    - The frontend only trims; the backend falls back to English for an
      empty field (Step 2), and the fields are refreshed from the command's /
      event's answer so the user sees the stored value.
    - Keep the preset strings in one place: fetch them from the backend (via
      the `label_names` response or a dedicated command) or duplicate the
      five strings in the helper module; note the choice in `learnings.md`.

- [ ] Step 4: README, docs/usage.md, CLAUDE.md and the todo item
  - Done when:
    - README "Working with other software" names the formats "Lightroom
      (XMP)" and "PhotoLab (.dop)" as the settings do, and gains a
      "Lightroom Classic" subsection: LrC does not write XMP by default
      (`Ctrl+S` / Catalog Settings > "Automatically write changes into
      XMP"); after import, LrC only re-reads a sidecar Riffle changed through
      Library > Synchronize Folder with "Scan for metadata updates"
      (restarting LrC does not re-read); import reads the XMP on first
      import; colour labels need either LrC's label set named
      Red/Yellow/Green/Blue/Purple or Riffle's label name setting matching
      the names in LrC's set (Japanese default preset provided).
    - README "Sidecar formats and software" adds `Adobe Lightroom Classic
      2026 (Windows, Japanese UI)` under XMP, ticked only once the user
      confirms stars, flags and labels all show after this feature (say so in
      the PR if left unticked); the existing Lightroom 9.5.1 line stays
      unticked.
    - `docs/usage.md` "Ratings and sidecars" says `xmp:Label` carries the
      configured name (English by default) and `photoshop:LabelColor` the
      lowercase English colour, and that a sidecar whose `xmp:Label` matches
      a configured or English name is shown in that colour.
    - `CLAUDE.md`'s layout sentence mentions the configurable `xmp:Label`
      names (`labelNames` key) next to `sidecarFormat`.
    - `todo.md`'s "App: the Lightroom 9.5.1 round-trip check is still open"
      item records the answer (Lightroom Classic 2026 does not resolve the
      colour from `LabelColor`; the write approach was revised by this
      plan), ticks / removes the `LabelColor` sub-item, and keeps the folder
      open check (`D:\Photos\2026\2026-09-05`) only if the user has not run
      it; the wrap-up's todo curation may then close the heading.
    - `mise run ci` (lychee link check included) passes.
  - Implementation approach:
    - Assumes Step 3 is merged. Docs only.

## Trade-offs and risks

- **Where the name mapping lives**: in `riffle-core`'s `xmp.rs` (chosen:
  the XMP semantics and their tests stay in one file, `dop.rs` is untouched)
  vs. an app-side wrapper that translates canonical names before / after
  the unchanged core functions (smaller core diff, but the mapping and the
  raw-string fallback then live away from the parser).
- **Names reaching the writer thread**: a snapshot per `Message::Set`
  beside `format` (chosen: consistent with the format, no shared lock in
  the thread) vs. an `Arc<Mutex<LabelNames>>` the thread reads at write
  time (fewer signature changes, but a judgement queued before a change is
  written with the new names).
- **What a names change does to the open folder**: (a) reset the index's
  sidecar parse state (`Index::reset_sidecars`) and emit `sidecar-format`
  so `main.ts` (line ~1909) reopens the folder and labels written with the
  new names are read back, at the cost of re-parsing the sidecars
  (thumbnails are kept); (b) only future reads / writes use the new names,
  and sidecars already parsed keep their label until their mtime changes.
  (a) is recommended for correctness; (b) is simpler. Under (a) the command
  must be `async` + `spawn_blocking`.
- **Reading back after switching names**: matching configured + English
  names only means a file written under the Japanese names reads as raw
  "レッド" (grey) once the user resets to English. Always matching the
  Japanese preset too would avoid that but hard-codes one locale into the
  reader. Not taken.
- **Empty or duplicate names**: an empty field falls back to English (LrC
  cannot match an empty name); duplicates are not rejected, the first
  colour in Red..Purple order wins on read. Rejecting duplicates in
  `set_label_names` is an option if stricter validation is wanted.
- **Which Lightroom was verified**: the experiments were on Lightroom
  Classic 2026; the todo item and README checklist name Lightroom desktop
  9.5.1 (not Classic), which stays unverified. The todo's first sub-item
  (opening the reference folder under XMP and checking flags / labels /
  stars) is also not answered by the experiments; Step 4 keeps it unless
  the user confirms.
- **Out of scope, open question**: LrC writes a pick as
  `xmpDM:good="true"` + `xmpDM:pick="1"`, unflagged as `xmpDM:pick="0"`,
  and presumably a reject as `xmpDM:pick="-1"` (unverified). Riffle only
  reads and writes `xmpDM:good`; whether a Riffle-written sidecar without
  `xmpDM:pick` shows the flag in LrC is unverified and left for a
  follow-up (a `todo.md` item if the user wants it tracked).

## Progress

- (2026-09-23) Step 1 complete
- (2026-09-23) Step 2 complete
