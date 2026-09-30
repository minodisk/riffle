<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Clear three fully-specified todo.md items

## Purpose

`todo.md` carries three self-contained items whose fix is already spelled
out: the shared Exif reader leaves the trailing spaces old FinePix bodies
pad `Model` with, two frontend callers of `tree.ts`'s path helpers stay
case-sensitive on macOS / Windows, and no guide under `docs/agents/`
covers GitHub Actions workflows or `gh` secrets. Each step closes one
item and edits its `todo.md` section in the same PR, so the list shrinks
to what still needs real work.

Every step touches `todo.md`, so run the steps one after another, each
branched from `main` after the previous PR merged, rather than in
parallel.

## Steps

- [x] Step 1: Trim trailing spaces from the Exif `Make` / `Model` in the shared reader
  - Done when: `crates/core/src/exif.rs` returns `make` and `model`
    without trailing spaces (and without the NUL terminator, as today);
    a unit test in `exif.rs`'s `tests` module feeds a padded value (e.g.
    `FinePix E550   `) through `read_ifd0` for both byte orders and
    asserts the trimmed text; the now-redundant local trim in `orf.rs` is
    removed and its test still passes; `cargo test -p riffle-core` and
    `mise run ci` pass; the `### Core: old FinePix bodies show Exif
    `Model` with trailing spaces` section of `todo.md` keeps only the
    real-device check (its TODO line becomes "check the meta pane on a
    FinePix RAF shows the trimmed `Model`", with the Background noting
    the reader now trims); `learnings.md` records which parsers the trim
    covers and which do not go through it.
  - Implementation approach:
    - `exif::ascii` (`crates/core/src/exif.rs` ~line 93) already does
      `trim_end_matches('\0')`; `read_ifd0` (~line 50) assigns
      `shot.make = ascii(tiff, e)` / `shot.model = ascii(tiff, e)`. Trim
      in `read_ifd0` for those two tags only (e.g.
      `ascii(tiff, e).map(|s| s.trim_end().to_string())`), not inside
      `ascii`, so `DateTimeOriginal`, `SubSecTimeOriginal` and
      `LensModel` keep their bytes.
    - The test helper `crate::jpeg::tests::W` (`w.ascii(tag, s)` appends
      the NUL) builds the TIFF; follow the existing
      `ifd0_and_exif_ifd_fed_from_two_tiffs` test in `exif.rs`.
    - Coverage: `read_ifd0` is called by `jpeg.rs` (plain JPEG, and RAF
      through `jpeg::read_exif`), `nef.rs`, `cr3.rs` and `orf.rs`. It is
      **not** used by `arw.rs`, whose own `ascii` (~line 310, ARW and
      DNG) trims NULs only; leave `arw.rs` alone (no padded Sony / Leica
      / Sigma `Model` is known) and note this in `learnings.md`.
    - Remove the `orf.rs` local trim of `make` / `model` (~lines 51-54,
      "Olympus pads Make and Model with spaces") that follows
      `exif::read_ifd0`; its test (`orf.rs` ~line 394) keeps asserting the
      trimmed `OM Digital Solutions` / `OM-1`.
    - `docs/agents/raw-metadata-parsing.md` (~line 389) says "The Exif
      `Model` of many older FinePix bodies carries trailing spaces ...
      so trim it before comparing or grouping by model"; reword that
      sentence to say the shared reader (`exif::read_ifd0`) trims it,
      so a reader does not add a second trim.

- [x] Step 2: Pass the platform `ignoreCase` flag to `main.ts`'s rename rebase and `trash.ts`'s `relation` calls
  - Done when: on macOS / Windows, `renameFolder` in
    `crates/app/ui/src/main.ts` reopens an `openDir` that differs only
    in case from the renamed path, and `restoredInto` / `opensTarget`
    in `crates/app/ui/src/trash.ts` match an `openDir` that differs
    only in case; `crates/app/ui/src/trash.test.ts` has cases where
    `restoredInto` and `opensTarget` are given a differently cased
    `openDir` and match with the flag on and not without it;
    `pnpm exec vp test` and `mise run ci` pass; the `### App:
    `main.ts` / `trash.ts` path comparisons stay case-sensitive on
    case-insensitive filesystems` section of `todo.md` is removed.
  - Implementation approach:
    - `folders.ts` line 159 holds
      `const ignoreCase = isMac || /Win/.test(navigator.platform)`;
      export it (`export const ignoreCase`). `main.ts` already imports
      from `folders`, so it can import the flag from there.
    - `trash.ts` must **not** import `folders.ts`: `folders.ts` runs
      `document.getElementById("folders")` at module load (line 58) and
      `trash.test.ts` runs under Vitest `environment: "node"`
      (`vite.config.ts`). Instead give `restoredInto(openDir, restored,
      allFiles, ignoreCase = false)` and `opensTarget(openDir, dirs,
      recursive, ignoreCase = false)` a trailing parameter, defaulting
      to `false` exactly as `tree.ts`'s `relation` / `rebase` /
      `renameFolder` do, and forward it to `relation(...)` (lines 110
      and 128). `main.ts` passes the flag at its three call sites
      (`opensTarget` at ~line 722 and ~1581, `restoredInto` at ~1582).
    - In `main.ts`'s `renameFolder` (~line 2430) pass the flag to both
      `rebase` calls: the `moved` helper that remaps the trash
      history's folders (`rebase(dir, path, newPath, ignoreCase)`) and
      the `reopen` line (`rebase(openDir, path, newPath, ignoreCase)`),
      so the trash history stays consistent with the reopened folder.
    - Tests: `tree.test.ts` already covers `rebase(..., true)` (~line
      697) and `relation(..., true)` (~line 674), so the new cases go in
      `trash.test.ts` next to the existing `restoredInto` /
      `opensTarget` describes (lines 91 and 127), e.g.
      `opensTarget("d:\\photos\\2026", ["D:\\Photos\\2026"], false, true)`
      is `true` and the same call without the flag is `false`;
      `restoredInto("c:\\Photos", ["C:\\photos\\1.ARW"], [], true)`
      returns the file. `main.ts` is not unit-tested; its rename path is
      verified by `pnpm exec vp check` (types) and reading.
    - Keep the change to these three files plus the test and `todo.md`;
      no other caller of `relation` / `rebase` changes.

- [ ] Step 3: Add `docs/agents/github-actions-workflows.md`
  - Done when: the guide exists, follows the shape of the existing
    guides (title, a "Read this before touching `.github/workflows/*.yml`
    that uses `gh`, secrets, or a dedicated branch as a data store"
    opener, the Hit / Measured / Inferred tags with the reference to
    `tauri-app.md`, a `Source:` line), covers the four points below, links
    `docs/plans/_archived/20260928-promotion-stats/learnings.md`,
    `mise run ci` (lychee included) passes, and the `### Docs: add a
    guide for GitHub Actions workflows and `gh` secrets` section of
    `todo.md` is removed. `CLAUDE.md` is not changed (no guide index
    exists; the other guides are found by listing `docs/agents/`).
  - Implementation approach:
    - Points, each with its reason, taken from the archived
      `learnings.md` ("After the Step 2 merge" and "Step 1"):
      1. An empty secret is indistinguishable in `gh secret list`; the
         tell is an empty `GH_TOKEN:` line in the run log (and gh's
         "set the GH_TOKEN environment variable" error). Set secrets on
         github.com, or check the value is non-empty before
         `gh secret set` (the `read -rs T && printf '%s' "$T" | gh
         secret set` Git Bash recipe registered an empty value). Tag:
         Hit.
      2. Pushing a branch that adds or changes `.github/workflows/*`
         needs the `workflow` scope on the gh token
         (`gh auth refresh -h github.com -s workflow`); otherwise the
         push is refused with "refusing to allow an OAuth App to create
         or update workflow". Tag: Hit.
      3. `shellcheck` / `actionlint` are pinned in `mise.toml` and run
         by the lint task, but are not on the Git Bash `PATH` outside
         mise tasks; run them ad hoc with `mise exec -- shellcheck ...`
         / `mise exec -- actionlint`. Tag: from the learnings wording.
      4. A dedicated branch as a data store: `git worktree add --orphan
         -b <branch> <dir>` (git 2.42+, present on `ubuntu-latest`) for
         the first run, `git fetch --depth=1 origin <branch>:<branch>`
         plus `git worktree add` afterwards, and `git diff --cached
         --quiet` as the commit-only-when-changed guard. Point at
         `.github/workflows/stats.yml` as the live example. Tag: Measured.
    - Link the learnings as
      `../plans/_archived/20260928-promotion-stats/learnings.md`
      (relative to `docs/agents/`, as the other guides' `Source:` lines
      do); lychee checks it.

## Trade-offs and risks

- **Step 1: where to trim.** Decided: trim `Make` / `Model` in
  `read_ifd0` only, not in `exif::ascii`, so `capture_time` strings the
  sequencer compares are unchanged.
- **Step 1: the redundant `orf.rs` trim.** Decided: remove it, so one
  place trims.
- **Step 1: `arw.rs` stays untrimmed.** ARW / DNG have their own ASCII
  reader. Not touched here (no known padded body).
- **Step 2: parameter vs shared module.** Decided: thread `ignoreCase`
  through `restoredInto` / `opensTarget` as a defaulted trailing
  parameter, keeping `trash.ts` pure and testable under Node.
- **Step 2: the `moved` rebase in `renameFolder`.** Decided: pass the
  flag to both `rebase` calls.
- **Step 3: `CLAUDE.md` registration.** Decided: none, per convention.
- **Ordering.** All three steps edit `todo.md`; run them sequentially
  from a fresh `main`.

## Progress

- (2026-10-01) Step 1 complete
