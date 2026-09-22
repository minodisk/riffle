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

# Lightroom XMP flags and colour labels

## Purpose

Lightroom 9.5.1 (Windows, Japanese UI) writes its pick / reject flag as
`xmpDM:good="True"` / `"False"` (namespace
`http://ns.adobe.com/xmp/1.0/DynamicMedia/`) on the `rdf:Description`,
absent when unflagged, and keeps `xmp:Rating` as the star count even on a
reject; it never uses `xmp:Rating="-1"`. It writes the colour label twice:
a localised `xmp:Label` (`"パープル"`) and a language-independent
`photoshop:LabelColor="purple"` (namespace
`http://ns.adobe.com/photoshop/1.0/`).

Riffle folds the reject into the rating (`-1`), has no pick under XMP, and
reads the colour from `xmp:Label` alone, so a Lightroom-flagged folder opens
with no picks or rejects, a rejected shot loses its stars once Riffle
rewrites it, and localised labels render as "other" (grey).

This work widens Riffle's judgement to a tri-state flag (none / pick /
reject) independent of the `0`-`5` stars, the way both Lightroom and DxO
PhotoLab model it; reads and writes `xmpDM:good` and `photoshop:LabelColor`;
and lets a pick be kept under XMP. Afterwards, opening
`/mnt/d/Photos/2026/2026-09-05` with the XMP format shows L1005439 picked,
L1005438 rejected and L1005428-L1005432 with purple / blue / green / yellow /
red labels, and picks, rejects, stars and labels given in Riffle round-trip
to Lightroom whatever its UI language.

Reference sidecars (read-only; trim into fixtures, never edit in place):
`/mnt/d/Photos/2026/2026-09-05/L1005428.xmp` .. `L1005439.xmp`.

Decisions already taken (do not reopen):

- Flag is a tri-state separate from the rating; a reject keeps its stars.
- XMP write shape is exactly Lightroom's: `xmp:Rating="<stars>"` plus
  `xmpDM:good="False"` for a reject, `xmpDM:good="True"` for a pick, no
  `xmpDM:good` when unflagged. A legacy `xmp:Rating="-1"` (darktable /
  Bridge) reads as a reject with 0 stars.
- Labels: write both `photoshop:LabelColor` (lowercase) and the English
  `xmp:Label`; read `LabelColor` first and fall back to `xmp:Label` only when
  `LabelColor` is absent.

## Steps

- [x] Step 1: Tri-state flag in `riffle-core`: `xmpDM:good` in `xmp.rs`, stars kept on a `.dop` reject
  - Done when:
    - `crates/core/src/lib.rs` exports a `Flag { None, Pick, Reject }` enum
      (`Copy`, `Default = None`) shared by `xmp.rs` and `dop.rs`.
    - `xmp::read_flag(bytes) -> Result<Flag, String>`: `Pick` for
      `xmpDM:good="True"`, `Reject` for `"False"` or for a legacy
      `xmp:Rating="-1"`, `None` otherwise (attribute and element forms;
      value compared case-insensitively, trimmed).
    - `xmp::read_rating` returns only `0..=5` (`-1` reads as `None`; the
      reject it meant is reported by `read_flag`).
    - `xmp::write_rating(existing, rating: Option<i8>, flag: Flag)`: writes
      `xmp:Rating` as before (`None` -> `0`, range `0..=5`, `-1` is an
      error), and `xmpDM:good="True"` / `"False"` for `Pick` / `Reject`,
      removing the attribute or element for `None`. Patching splices only
      the value when present, inserts one attribute (declaring
      `xmlns:xmpDM` on the first `rdf:Description` when no prefix is bound
      to the namespace) when absent, and leaves every other byte
      untouched. A legacy `xmp:Rating="-1"` being rewritten becomes the
      stars given (`0` when none). The fresh template declares `xmlns:xmpDM`
      only when it writes `good`.
    - `dop::read_rating` returns `Rating` (`0..=5`) regardless of
      `ShouldProcess`; `dop::read_flag(bytes) -> Result<Flag, String>`
      maps `ShouldProcess` `0` / `1` / other; `dop::write_rating(existing,
      rating, flag, name, now)` always writes `Rating = <stars>` and
      `ShouldProcess = 0 / 1 / 2`; the template does the same (no more
      `(-1, _) => (0, 1)` arm). `read_pick` is removed.
    - `crates/app/src/sidecar.rs` `SidecarFormat` is adapted just enough to
      compile and keep today's app behaviour: `read_pick` maps
      `read_flag() == Pick`, and a temporary shim in `write_rating` turns
      the app's `(rating, pick)` into `(rating.filter(|r| *r != -1),
      if rating == Some(-1) { Reject } else if pick { Pick } else { None })`.
      Under XMP the shim also reports a `Reject` flag as `rating = Some(-1)`
      on read so the UI keeps working until Step 3 replaces it. Mark the
      shim with a comment naming Step 3.
    - Unit tests in `xmp.rs` cover: a trimmed Lightroom-shaped attribute
      fixture (one `rdf:Description` declaring `xmlns:xmp`,
      `xmlns:photoshop`, `xmlns:xmpDM`, with `xmp:Rating="0"` and
      `xmpDM:good`), the element form, pick -> reject -> none transitions
      leaving every other byte untouched, a reject keeping `Rating="3"`,
      legacy `-1` reading as `Reject` + `None` stars and being rewritten
      to Lightroom's shape, a `xmpDM` prefix bound to another namespace,
      and the fresh template for each flag. Tests in `dop.rs` cover a
      rejected item keeping its stars on read and write. Existing tests
      updated for the new signatures.
    - `mise run ci` passes.
  - Implementation approach:
    - Generalise `locate(text, name)` to take the namespace, and
      `xmp_prefix` / `resolve_prefix` to take the namespace plus its
      preferred prefixes (`["xmp", "xap"]` for XMP, `["xmpDM"]` here,
      `["photoshop"]` in Step 2), keeping the "reuse a bound prefix,
      declare an unbound one, skip one bound elsewhere, else `xmpN`" rule.
      `set`, the removal path (today only in `write_label`) and `template`
      need the same parameterisation; `template` must emit more than one
      attribute and declaration for a fresh rated + flagged sidecar.
    - Update the module docs of both files (`xmp.rs` still says `-1` is a
      reject and only `Rating` / `Label` are written).
- [x] Step 2: `photoshop:LabelColor` in `crates/core/src/xmp.rs`
  - Done when:
    - `xmp::read_label` returns the colour of a non-empty
      `photoshop:LabelColor`, normalised to the capitalised English name
      the UI uses (`"purple"` -> `"Purple"`), and falls back to the raw
      `xmp:Label` only when `LabelColor` is absent or empty.
    - `xmp::write_label(existing, Some(name))` writes both
      `photoshop:LabelColor` (lowercased `name`) and `xmp:Label` (`name`,
      English); `write_label(existing, None)` removes both. Each is
      spliced in place when present and inserted (declaring
      `xmlns:photoshop` when needed) when absent; the fresh template
      carries both.
    - Unit tests: the Lightroom fixture with `xmp:Label="パープル"` +
      `photoshop:LabelColor="purple"` reads `"Purple"`; relabelling it to
      Red rewrites exactly the two attributes; clearing removes both; a
      Bridge-style fixture with only `xmp:Label` still reads and is patched
      as before (now gaining `LabelColor`); the element form for
      `LabelColor`; existing label tests updated for the new write shape.
    - `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (namespace-parameterised helpers).
    - The UI toggles with a case-sensitive `focused.label === "Purple"`
      (`crates/app/ui/src/main.ts`), while `strip.ts` and `filter.ts`
      lowercase, so normalising the read value keeps the toggle, tint and
      filter working with no UI change.
    - Orange and Pink are outside Lightroom's five; still write
      `LabelColor="orange"` / `"pink"` (see Trade-offs).
- [ ] Step 3: Tri-state flag through the app: index migration, commands, writer, UI state
  - Done when:
    - `ratings` gains `flag INTEGER NOT NULL DEFAULT 0` (`0` none, `1`
      pick, `2` reject) and loses `pick`; `SCHEMA_VERSION` becomes 11 with
      an in-place migration that keeps every row, dirty ones included:
      `flag = CASE WHEN rating = -1 THEN 2 WHEN pick = 1 THEN 1 ELSE 0
      END`, then `rating = NULL WHERE rating = -1`, then `DROP COLUMN
      pick`. The `files` table is not dropped (no rescan). A migration test
      seeds a v10 database with a dirty `-1` row, a picked row and a plain
      row and checks the three flags and that `rating` no longer holds
      `-1`. The schema-history doc comment gains the v11 sentence.
    - `IndexedFile`, `ParsedSidecar`, `dirty_rows`, `set_rating`,
      `mark_written`, `store_sidecar_ratings`, `reset_sidecars` and the
      sidecar `Judgement` carry `flag: Flag` instead of `pick: bool`;
      `rating` is `0..=5` everywhere. The Step 1 shim in `sidecar.rs` is
      deleted; `SidecarFormat::read_flag` / `write_rating(.., flag)` call
      through for both formats with no `Dop` gate, and `write()`'s
      "nothing to write" test becomes `rating.is_none() && flag == None
      && label.is_none()`.
    - `#[tauri::command] set_rating(path, rating: u8 0..=5, flag: "none" |
      "pick" | "reject", label, label_known)`; the `format == Dop` gate is
      gone; `folder_entries` serialises `flag` as the same strings.
    - UI: `picks: Set` becomes `flags: Map<string, "pick" | "reject">` in
      `main.ts` and `strip.ts`; `ratings` holds only `1`-`5`; `Judgement`
      (`main.ts`, `filter.ts`), undo / redo, `rejectRest`, `trash.ts`,
      `applyRating`, `strip.setRating` and `paintRating` use `flag`. Key
      semantics: `x` sets reject and keeps the stars (replacing a pick);
      `p` sets pick and keeps the stars (replacing a reject); `u` clears
      the flag only; `0` clears the stars only; `c` clears both and the
      label; `1`-`5` keep the flag. A rejected cell stays dimmed and now
      shows its stars; the filter derives `flag` from `flag` and `stars`
      from `rating ?? 0`. `effectivePick` and the XMP gates in `main.ts` /
      `context.ts` are removed along with `pick.ts` / `pick.test.ts`.
    - Rust tests (`index.rs`, `sidecar.rs`, `commands.rs`, including
      `xmp_ignores_a_pick` -> a pick reaches the XMP,
      `a_photolab_pick_is_read_with_dop_and_ignored_with_xmp` -> a
      Lightroom `.xmp` with `xmpDM:good="True"` reads as a pick) and
      vitest suites (`filter`, `trash`, `undo`, `context`) updated;
      `mise run ci` passes.
    - Manual check: opening `/mnt/d/Photos/2026/2026-09-05` under XMP
      shows 439 picked, 438 rejected, 428-432 purple / blue / green /
      yellow / red, 433-437 with 5..1 stars. On a copy of the folder,
      `p` / `x` / `u` / `3` / a colour key patch only the expected
      attributes (diff the sidecar before and after) and a reject given on
      a starred file keeps `xmp:Rating`.
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged.
    - One PR because the IPC shape (`pick` -> `flag`, no `-1`) couples
      backend and frontend; keep it to the model change and do not fold
      docs in (Step 4).
    - `set_rating` keeps writing the row before telling the writer, and
      `reconcile_sidecars` / `store_sidecar_ratings` keep their `dirty`
      snapshot rule (see `docs/agents/tauri-app.md`).
    - `README`-visible wording is left for Step 4; only code and tests
      here.
- [ ] Step 4: Documentation: README, CLAUDE.md
  - Done when:
    - README "Working with other software" no longer says XMP "cannot hold
      a pick"; the key table reads: `0` clears the stars, `x` reject (keeps
      the stars), `p` pick (no "PhotoLab format only"), `u` clears the pick
      / reject, `c` clears everything; the "Move Rejected to Trash" text
      still holds.
    - The "Sidecar formats and software" checklist names the software
      actually verified — Lightroom desktop 9.5.1 (Windows), which is not
      Lightroom Classic — and ticks it only once the user confirms the
      round-trip in Lightroom (say so in the PR if left unticked).
    - `CLAUDE.md`'s `xmp.rs` / `dop.rs` sentence mentions the tri-state
      flag, `xmpDM:good` and `photoshop:LabelColor`.
    - The Step 4 commit carries a `Release-As: 0.3.0` footer so
      release-please cuts 0.3.0 (the user wants a minor bump for Lightroom
      support; `bump-patch-for-minor-pre-major` would otherwise give 0.2.1).
  - Implementation approach:
    - Assumes Step 3 is merged. Docs only.

## Trade-offs and risks

- **Tri-state `flag` column replacing `pick` vs adding a `reject` column
  beside it**: a second bool needs fewer renames but leaves two mutually
  exclusive bools to keep consistent in every UPDATE; the plan picks one
  `flag` column and a `Flag` enum end to end. Dropping `pick` needs SQLite
  >= 3.35 (`DROP COLUMN`), which rusqlite 0.40 bundles.
- **Step 3 size**: the IPC change forces backend and UI into one PR. The
  alternative (backend serving both `pick` and `flag` for one release) adds
  throwaway code; not taken.
- **Lightroom reading the English `xmp:Label`**: Japanese Lightroom will
  see `Label="Red"` beside `LabelColor="red"`. Whether 9.5.1 resolves the
  colour from `LabelColor` when `Label` does not match its localised set is
  unverified; only the user's Lightroom check can tell. If it does not, the
  fallback is to leave an existing localised `xmp:Label` untouched when the
  colour is unchanged (revisit in a follow-up).
- **Orange and Pink**: not in Lightroom's five; `LabelColor="orange"` /
  `"pink"` are Riffle's extrapolation and may show as a custom label in
  Lightroom. Documented, not blocking.
- **Legacy `-1` sidecars**: after Riffle rewrites one, `xmp:Rating` becomes
  `0` and `xmpDM:good="False"` is added, so darktable / Bridge no longer
  see the reject. Accepted by decision 2.
- **`.dop` reject now writes `Rating`**: today a reject leaves `Rating` as
  is; after Step 1 it writes the stars Riffle holds. Between Step 1 and
  Step 3 the shim passes `None` stars on a reject, i.e. `Rating = 0`.
  Merge Steps 1-3 in quick succession.
- **Old undo entries / touched state**: per-folder and in-memory, nothing
  persisted; no migration needed on the UI side.

## Progress

- (2026-09-22) Step 1 complete
- (2026-09-22) Step 2 complete
