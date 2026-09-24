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

# Write both sidecars: a "Both" sidecar format

## Purpose

`SidecarFormat` (`crates/app/src/sidecar.rs`) is XMP or `.dop`, one at a time:
the other format's files are neither read nor written. A user who develops in
both Lightroom and DxO PhotoLab has to switch formats and re-cull. A third
choice, "Both", writes every judgment (rating, pick / reject flag, color label)
to `FOO.xmp` and `FOO.ARW.dop` together, and reads back whichever of the two
was modified last, so an edit made later in either Lightroom or PhotoLab is
picked up. Once done, one culling pass serves both developers.

## Steps

- [x] Step 1: Add the `Both` sidecar format end to end (backend, settings UI, first-launch dialog, docs)
  - Done when:
    - `SidecarFormat::from_setting(Some("both"))` is `Both`, `Both.setting()`
      is `"both"`; `"xmp"`, `"dop"`, missing and unknown values behave as
      before (unknown / missing -> `Xmp`).
    - In `Both` mode the writer thread writes each judgment to the XMP and
      the `.dop` sidecar, each through the existing temp + fsync + rename
      path. `label_known = false` keeps the label the newest existing sidecar
      holds and writes that label to both files. Clearing everything on a file
      that has neither sidecar creates neither; a file that has only one
      sidecar gets that one patched and, when the judgment is not all-clear,
      the other one minted.
    - Reading in `Both` mode (folder open / rescan through
      `reconcile_sidecars_of`): when both sidecars exist and disagree, the one
      with the newer mtime wins; when only one exists, it is used; when neither
      exists a clean row is cleared as today. The stat stored in
      `ratings.xmp_size` / `xmp_mtime_ns` after a write is the one the read
      path will compare against, so a folder reopened right after culling
      re-parses nothing.
    - The folder watcher still ignores the app's own writes of both files
      (already true: `watch::triggers` drops `*.xmp` and `*.arw.dop` of either
      format regardless of the setting; verify, and only touch it if a test
      shows otherwise).
    - `set_sidecar_format` / `choose_sidecar_format` accept `"both"`; switching
      Xmp -> Both, Dop -> Both, Both -> Xmp and Both -> Dop goes through the
      unchanged `switch_format` (drain, persist, `reset_sidecars`), and the
      next folder open reads the newly selected format(s) and writes dirty rows
      into them.
    - The settings window's Sidecar tab and the main window's first-launch
      dialog each offer three choices (Lightroom (XMP) / DxO PhotoLab (.dop) /
      Both), the label-names block in the settings window is shown for `xmp`
      and `both` (both write `xmp:Label`), and `showSidecarFormat("both")`
      checks the right radio.
    - Unit tests: `from_setting("both")` and `setting()` round trip; a judgment
      in `Both` mode lands in both files; clearing with no sidecar creates no
      file; `label_known = false` keeps the newest sidecar's label in both
      files; reconcile in `Both` mode with two disagreeing sidecars takes the
      newer mtime (both orders); reconcile with only the XMP, and with only the
      `.dop`, reads that one. Existing tests pass; `mise run ci` is green.
    - `README.md`, `README.ja.md` (same PR), `docs/usage.md` and the
      `CLAUDE.md` layout sentence ("the XMP-or-`.dop` setting") describe the
      third choice; Lightroom is listed before PhotoLab wherever both appear.
  - Implementation approach:
    - **Enum** (`crates/app/src/sidecar.rs`): add `SidecarFormat::Both`
      (setting value `"both"`), keep `#[default] Xmp`. Add a `kinds()` helper
      that yields the concrete file kinds a setting covers (`[Xmp]`, `[Dop]`,
      `[Xmp, Dop]`) and use it everywhere the code iterates "the sidecar(s) of
      this setting". `matches` on `Both` is "either kind". **Decided with the
      user: keep one enum.** The per-file methods (`sidecar_path`,
      `read_rating`, `read_flag`, `read_label`, `write_rating`, `write_label`)
      get a `Both` arm that is `unreachable!`, documented as "call `kinds()`
      first"; no code path may call a per-file method with `Both` at runtime,
      and the tests are what guard it.
    - **Newest-wins rule, one helper**: write a single function that picks
      the effective sidecar among the existing kinds by the larger
      `mtime_ns`, with a deterministic tie-break (XMP before `.dop`, the
      order the kinds are listed in), and use it in *both* places: in
      `sidecar::write` (to decide which file's label to keep when
      `label_known` is false, and which file's `(size, mtime_ns)` to hand to
      `Index::mark_written`) and in `commands::reconcile_sidecars_of` (to
      decide which file to compare against the stored stat and parse). Using
      one comparator in both places is what keeps the stored stat and the
      read path's pick consistent; an exFAT card stores mtimes at 2 s
      resolution, so ties are real, and a tie broken differently on the two
      sides would re-parse every open.
    - **Writer** (`sidecar::write`): loop over the kinds, running the existing
      per-kind body (existing-sidecar lookup with the case-insensitive
      fallback, the "all clear and no file -> skip" rule, patch or mint, temp
      + fsync + rename) once per kind. A failure in either kind fails the
      whole entry so the existing retry re-runs both (patching is idempotent);
      note in a comment that the two files are each atomic, not atomic
      together. `Message::Set` / `Entry` keep carrying the `SidecarFormat`
      selected at judgment time, unchanged.
    - **Index** (`crates/app/src/index.rs`): no schema change and no
      `EXTRACTOR_VERSION` bump. `xmp_size` / `xmp_mtime_ns` already hold the
      stat of "the sidecar Riffle last saw" for `.dop` too; in `Both` mode
      they hold the newest one's stat. `has_sidecar`, `mark_written`'s
      guard, `reset_sidecars` and `store_sidecar_ratings` stay as they are.
      Update the `xmp_size` doc comment near the schema to say "the effective
      sidecar (the newest one under Both)".
    - **Folder open** (`crates/app/src/commands.rs`): `list_dir` collects the
      entries matching any kind of the setting (the map is keyed by
      lower-cased file name, so `foo.xmp` and `foo.arw.dop` coexist).
      `reconcile_sidecars_of` builds the `(path, Option<SidecarStat>)` pairs by
      looking up each kind's expected name and applying the newest-wins
      helper; when parsing `to_parse` it needs the kind of the chosen file,
      derived from the path (`SidecarFormat::Dop.matches(name)` else XMP) or
      carried alongside `SidecarStat`, to call the right reader.
      `scan_folder`, `set_rating` and the dirty-row replay pass the setting
      through unchanged.
    - **Format switch**: `switch_format` / `choose_format` need no logic
      change; add a test in `commands.rs`'s tests mirroring the existing
      Xmp -> Dop switch test for a switch to `Both` (dirty row written to both
      files after the switch).
    - **Tests**: `sidecar.rs` tests follow the existing `judge(...)` /
      `eventually(...)` helpers and read back with `xmp::read_*` /
      `dop::read_*`. For the mtime-ordering tests in `commands.rs`, set the
      files' mtimes explicitly with `std::fs::File::set_modified` rather than
      sleeping.
    - **Frontend**: `crates/app/ui/settings.html` gains
      `<label><input type="radio" name="sidecar-format" value="both" /> Both (.xmp and .dop)</label>`
      after the two existing radios; `crates/app/ui/src/settings.ts` replaces
      the two `format !== "xmp"` checks with one "writes XMP" predicate that is
      true for `xmp` and `both`. `crates/app/ui/index.html`'s `#format-choices`
      gains `<button type="button" data-format="both">Both</button>` (the
      buttons are `flex: 1` in `style.css`, so three fit). `firstrun.ts` /
      `FormatGate` is format-agnostic and needs no change.
    - **Watcher / trash**: `watch::triggers` and `trash.rs` already handle
      both kinds independently of the setting; expect no change.
    - **Docs**: `README.md` / `README.ja.md` "Working with other software"
      (add a **Both** bullet after the two existing ones, and widen the
      first-launch sentence "Lightroom or DxO PhotoLab" to include both),
      `docs/usage.md`'s sidecar section, and `CLAUDE.md`'s layout sentence.
      Mention that under Both the sidecar modified last is the one read back.

## Trade-offs and risks

- **Enum shape (decided: one enum).** Splitting the setting from a new
  `SidecarKind { Xmp, Dop }` would make misuse a type error but renames ~40
  call sites; the user chose the smaller diff with `unreachable!` arms.
- **Single stored stat vs. one per kind.** Storing only the newest sidecar's
  stat means an external change to the *older* file whose mtime stays older
  than the other file (a copy that preserves timestamps, a clock skew) is not
  noticed until that file is touched again. Real edits in Lightroom or
  PhotoLab set a fresh mtime, so they are always the newest and are picked up.
  Tracking both stats would need two more `ratings` columns and a
  `SCHEMA_VERSION` bump; not worth it for the requirement as stated.
- **Both files are not written atomically together.** A crash between the
  XMP rename and the `.dop` rename leaves the two disagreeing; the row stays
  dirty (`mark_written` never ran), and until it is judged again newest-wins
  reads whichever got written. An in-session retry replays the judgment into
  both; once retries are exhausted, `mark_partial_write` records the stat of
  the sidecar that did get written (without clearing `dirty`), so the next
  open does not mistake it for an external edit and still replays the
  judgment into both. Say so in the `write` doc comment.
- **Disagreeing sidecars on entering `Both`.** Switching to Both on a folder
  with older sidecars of the other kind reads the newer of the two per file;
  nothing is rewritten until the user judges the file again, so the two files
  may keep disagreeing for unjudged files. Worth one sentence in the README
  bullet.

## Progress

- (none yet)
