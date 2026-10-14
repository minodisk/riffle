<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Per-file viewed negatives replace Culling Finished

## Purpose

The personal ranking (`riffle_core::rank::fit`) needs to know which unpicked
frames the user actually looked at and left behind. Today that signal is the
folder-level `Culling Finished` mark (`../_archived/20261012-image-embeddings/plan.md`
Step 7, auto-set by `../_archived/20261013-auto-culling-finished/plan.md`):
in a finished folder every `Flag::None` frame counts as "not chosen", in any
other folder none does. The mark is coarse (one unviewed frame keeps a whole
folder out, and a hand-set mark counts frames never shown), carries two
columns plus a manual-override rule, a menu item and an auto-marking flow,
and makes the eviction exemption depend on it.

This plan replaces it with the per-file signal the `views` table already
records: an unflagged frame counts as "not chosen" when that file itself was
viewed, wherever it sits. "Viewed" gains a dwell condition so frames flown
past while holding an arrow key do not count. The folder-level mark, its
columns, commands, menu item, auto-mark and docs go away, the 30-day
eviction exemption moves onto the folders the fit reads, and the background
refit the mark used to trigger moves to app startup and folder switches.

Decisions (user decisions on 2026-10-14):

- Drop `folders.culling_finished`, `folders.culling_manual`, the
  `culling_finished` / `set_culling_finished` / `finish_culling` commands,
  the folder tree's `Culling Finished` check and `viewed.ts`'s `allViewed`
  with its `finish_culling` call in `main.ts`.
- `fit`: an unflagged frame is a negative (weight 1) when its file was
  viewed; a reject is a negative (`REJECT_WEIGHT`) anywhere; an unflagged,
  unviewed frame is ignored, as today in an unfinished folder.
- Folders fed to the fit (`Index::rank_dirs`): the folders holding any
  `views` row or any reject (`ratings.flag = 2`). A folder with picks alone
  cannot form a pair, so it is not read.
- 30-day eviction: a folder that has any `views` row or any reject is
  spared by the age rule, the same predicate as `rank_dirs`; the size cap
  and Clear Cache keep ignoring it, as today. Since `views` is never pruned,
  such a folder is spared by the age rule for good; the user accepted this
  (the size cap and Clear Cache still reclaim it).
- Refit triggers: the background refit runs at app startup and whenever the
  user switches from one open folder to another, while `aiEnabled` is on,
  on the per-file training set above. The settings' `Refit Ranking` stays.
- Dwell: a file counts as viewed once its decoded preview has been on
  screen for `VIEW_DWELL_MS = 500` ms, measured from the `draw()` that
  puts the bitmap of the current `seq` up (the worker `message` handler),
  cancelled when `seq` moves on before then (next `show()`, folder open,
  the list emptying). A failed preview (`failPreview`) no longer counts: the
  user saw an error, not the photo, and such a file has no embedding to
  rank anyway. The constant lives in `viewed.ts`, not a setting.
- `aiEnabled` stays off by default; views are recorded whatever it is, as
  today.

## Steps

- [x] Step 1: Remove Culling Finished and fit on per-file views
  - Done when:
    - `crates/core/src/rank.rs`: `Labeled` loses `finished: bool` and gains
      `viewed: Vec<bool>` parallel to `flags`; `pairs` weighs `Flag::None`
      at 1 when `viewed[i]`, else skips it; `Flag::Reject` is unchanged.
      Module doc, `Labeled` / `fit` doc comments and the test helper
      `labeled(first, second, finished, sharper)` follow (per-frame
      `viewed`). Tests cover: unflagged + viewed -> a pair (negative);
      unflagged + not viewed -> no pair; a reject counts whether or not
      viewed (rename `a_fit_counts_rejects_anywhere_and_the_rest_in_finished_folders_only`
      accordingly); the existing prior / lean tests pass with `viewed`.
    - `crates/app/src/index.rs`: `SCHEMA_VERSION` is 23. `CREATE TABLE
      folders` loses both columns and their comments; the `(8..21)` and
      `(8..22)` `ADD COLUMN` guards go; new guards
      `(21..23).contains(&version)` -> `ALTER TABLE folders DROP COLUMN
      culling_finished` and `version == 22` -> `DROP COLUMN culling_manual`
      (SQLite's `DROP COLUMN` already works in the test fixtures, so the
      bundled SQLite is new enough); the accepted-version list gains 22.
      The schema doc comment's v21 / v22 sentences are rewritten and a v23
      sentence added. `culling_finished`, `set_culling_finished`,
      `finish_culling` are removed. `RankRow` gains `viewed: bool`, read by
      `rank_rows` through `LEFT JOIN views ON views.path = files.path`.
      `rank_dirs` returns `Vec<String>`:
      `SELECT DISTINCT dir FROM views UNION SELECT DISTINCT dir FROM ratings WHERE flag = 2 ORDER BY dir`.
      `evict` replaces `culling_finished` in its `SELECT` with the same
      predicate (one SQL fragment or helper shared with `rank_dirs`, so the
      exemption and the training set cannot drift apart) and the doc
      comments say "a folder the ranking reads (any viewed file or
      reject)".
    - Index tests: every v8+ fixture drops its
      `ALTER TABLE folders DROP COLUMN culling_finished;` /
      `... culling_manual;` lines (a regex pass, as `docs/agents/tauri-app.md`
      suggests); `a_v20_database_gains_culling_finished_and_keeps_its_folders`
      and `a_v21_database_gains_views_and_culling_manual_and_keeps_its_folders`
      are replaced by `a_v22_database_drops_the_culling_columns_and_keeps_its_folders`
      (fixture adds both columns back, `PRAGMA user_version = 22`, with a
      `last_viewed` and files set; asserts `SCHEMA_VERSION`, files,
      `last_viewed` kept, and `PRAGMA table_info(folders)` without the two
      columns) and a v21 fixture (only `culling_finished` added, `DROP TABLE
      views`); the v20 fixture still migrates.
      `culling_finished_is_set_on_any_folder_and_survives_a_rename` and
      `finish_culling_marks_an_untouched_folder_once_and_never_a_toggled_one`
      go. `rank_dirs` test: a folder with a view, one with a reject, one
      with a pick only and one with nothing -> the first two. `rank_rows`
      test gains the `viewed` column round trip.
      `the_age_rule_spares_a_finished_folder_and_clear_does_not` becomes
      `the_age_rule_spares_a_viewed_or_rejected_folder_and_clear_does_not`:
      an old folder with a `views` row and an old folder with a reject stay
      (rows and embeddings), an old folder with a pick only and an old
      untouched folder go; the size cap still removes a spared folder;
      `clear` drops everything.
    - `crates/app/src/rank.rs`: `refit` maps `rank_dirs` to `Labeled` with
      `viewed: rows.iter().map(|r| r.viewed).collect()`; `culling_finished`,
      `set_culling_finished`, `finish_culling` are removed (keep
      `refit_in_background` if Step 2 can reuse it, else remove it here and
      reintroduce what Step 2 needs); the module doc and
      `the_scores_match_rank_prior_py` (`finished: true` becomes
      `viewed: vec![true; n]`) follow. `crates/app/src/main.rs` drops the
      three handlers.
    - Frontend: `context.ts` `folderMenuGroups` loses the `finished`
      parameter and the `cullingFinished` group (and its comment);
      `context.test.ts` loses the "while AI analysis is on" describe and
      checks the signature; `main.ts` drops the `culling_finished` fetch,
      the `cullingFinished` case, `cullingFinishSent`, the `finish_culling`
      invoke and the `allViewed` import; `viewed.ts` / `viewed.test.ts`
      lose `allViewed`. `markViewed` still records a view (the dwell comes
      in Step 3), still skipping view-only folders and provisional resume
      anchors, and `viewedFiles` / `viewed_files` stay so a file is sent
      once per folder.
    - Docs in the same PR: `README.md` and `README.ja.md` Recommended order
      bullet (learns from the frames you viewed and did not pick);
      `docs/humans/usage.md` folder tree paragraph (the `Culling Finished`
      sentence goes) and **Recommended order** (what counts as not chosen,
      no mark, `Refit Ranking` is the refit for now, the cache keeps a
      folder with viewed frames or a reject past the month) with
      `usage.ja.md` in sync; `CLAUDE.md` layout (`index.rs` sentence: drop
      `culling_finished` / `culling_manual` / `finish_culling`, say `views`
      feeds the ranking's negatives and the age exemption; `rank.rs`
      sentence: drop `set_culling_finished` / `finish_culling`; `viewed.ts`
      sentence); `docs/agents/tauri-app.md` ranking bullets (rewrite the
      `Index::rank_dirs` sentence and the eviction sentence, replace the
      "`Culling Finished` is also set automatically" bullet with one on
      `views` feeding the fit per file) and a `SCHEMA_VERSION` guide line
      on dropping a column (fixtures faking v21 / v22 add the columns back
      instead of dropping them). `todo.md`: the ranking section's title and
      items that mark or check `Culling Finished` are rewritten around
      viewing frames + `Refit Ranking` and the eviction item around a viewed
      folder; the five auto-culling-finished items are replaced by checks
      that a viewed-and-unpicked frame moves the fit after `Refit Ranking`;
      the sidecar item loses "(after `Culling Finished` while AI analysis is
      on)"; the "no ranking refit when a finished folder's embeddings
      arrive" section is reworded to the current trigger set.
    - `mise run ci`, `cargo test -p riffle-core` and `cargo test -p riffle-app` pass.
  - Implementation approach:
    - Follow the `docs/agents/tauri-app.md` "Bumping `SCHEMA_VERSION`"
      entry: range-keyed guards keyed to the versions that had the column
      (v21 introduced `culling_finished`, v22 `culling_manual`), version
      asserts read `SCHEMA_VERSION`, audit every other guard.
    - Keep `views` exactly as is (table, `mark_viewed`, `viewed_files`,
      rename carry, eviction keeps it, `clear` drops it).
    - Keep `viewed_files` / `viewedFiles` in the frontend; without them
      every page turn would send a write, and Step 3's dwell timer uses the
      set to skip files already viewed.

- [x] Step 2: Refit at startup and on folder switch
  - Done when:
    - While `aiEnabled` is on, the app refits the ranking in the background
      (the same fit as `refit_ranking`, on the Step 1 per-file training set,
      storing `ranking.json`) once at startup, and once each time the
      frontend switches the open folder from one folder to another (not on
      the first open after launch, which startup already covers, and not on
      reopening the same folder). Nothing runs while `aiEnabled` is off.
    - A refit requested while one is running does not start a second fit in
      parallel (coalesce into at most one pending rerun, or skip; the
      implementer picks the simpler one that cannot lose the latest
      judgments for long and documents it).
    - The open folder's ranking in the UI picks up the new weights after a
      refit, the way it does after `Refit Ranking` today (reuse that path).
    - A unit test covers the "switch from A to B triggers, opening A first or
      reopening A does not" rule where it lives (a pure helper in the
      frontend, or the command side if the rule sits there), and the
      coalescing if it is testable without threads.
    - Docs in the same PR: `docs/humans/usage.md` / `usage.ja.md`
      **Recommended order** (the ranking relearns at startup and when you
      move to another folder; `Refit Ranking` relearns now); `README.md` /
      `README.ja.md` if the bullet mentions when it learns; `CLAUDE.md`
      `rank.rs` sentence (the startup / folder-switch refit);
      `docs/agents/tauri-app.md` ranking bullet on the triggers. `todo.md`
      gains manual checks: judging frames in folder A then opening folder B
      changes A's Recommended order on reopen without pressing `Refit
      Ranking`; restarting the app does too; nothing refits with AI
      analysis off.
    - `mise run ci` passes.
  - Implementation approach:
    - Startup: kick the refit from the app setup after the index and the
      `aiEnabled` setting are loaded, off the main thread and below the
      scan's priority concerns (it is a small fit, seconds at most).
    - Folder switch: the frontend knows the previous `openDir`; call a
      command (or reuse `refit_ranking` without awaiting) when it changes
      from a non-null folder to a different one.

- [x] Step 3: Count a file as viewed only after a dwell
  - Done when:
    - `crates/app/ui/src/viewed.ts` exports `VIEW_DWELL_MS = 500` and a
      `dwellTimer(onViewed, schedule = setTimeout, cancel = clearTimeout)`
      (or equivalent pure shape) with `shown(path)` starting one timer for
      the path on screen, `cancel()` stopping it, and a new `shown` for
      another path replacing it; `onViewed(path)` fires once per dwell.
    - `main.ts`: the worker `message` handler, after drawing the bitmap of
      the current `seq`, calls `dwell.shown(path)` (still skipping
      view-only folders, `openDir === null`, `pendingResume` and paths
      already in `viewedFiles`); `show()`, `openDirectory` and the
      `files.length === 0` branch of `refilter` call `dwell.cancel()`;
      `failPreview` no longer records a view; the dwell's `onViewed` is the
      old `markViewed` body (add to `viewedFiles`, `recordViewed`).
    - `viewed.test.ts`: a short display (`shown` then `cancel` or another
      `shown` before the dwell) records nothing; a display past the dwell
      records once; a second `shown` of the same path after recording does
      not re-fire (tested where that guard lives); a new path restarts the
      clock. Use injected `schedule` / `cancel` fakes or
      `vi.useFakeTimers`.
    - Docs in the same PR: `docs/humans/usage.md` **Recommended order**
      (a frame counts as viewed once it has been on screen for about half a
      second; frames flipped past while holding an arrow key do not count;
      a frame whose preview cannot be read does not count) with
      `usage.ja.md` in sync; `README.md` / `README.ja.md` if the bullet
      mentions viewing; `CLAUDE.md` `viewed.ts` sentence (the dwell rule
      and its constant); `docs/agents/tauri-app.md` `views` bullet (where
      the dwell is measured from and why a failed preview no longer
      counts). `todo.md` gains manual checks: holding `ArrowRight` across a
      fresh folder leaves only the dwelt-on frames in `views` (inspect
      `index.sqlite`), pausing on one frame for over half a second records
      it, and `Refit Ranking` afterwards reports the folder in the fit line.
    - `mise run ci` passes.
  - Implementation approach:
    - Measure from the bitmap's `draw()`, not from `show()`: the preview
      request and decode take 10-100 ms and a held key repeats every
      ~30 ms, so a timer started at `show()` would count frames whose
      bitmap never reached the screen.
    - Keep the batched `viewedWriter` as the write path; the dwell only
      gates when a path is pushed to it.

## Trade-offs and risks

- **Dwell threshold 500 ms (user choice, not a setting).** Held-key paging
  repeats every ~30 ms, so it never counts; comparing a burst at ~2 frames/s
  does count as judging those frames. Changing the constant later costs
  nothing in stored data (views are only ever added).
- **A failed preview no longer counts as viewed.** Such a file has no
  embedding, so it never enters a pair; the only effect is that a folder
  whose every file fails to preview is not spared by the age rule.
- **Eviction exemption = `rank_dirs` predicate (any view or any reject).**
  A pick-only folder gives the fit nothing (no negative to pair against),
  so sparing it keeps embeddings nobody reads.
- **The age rule becomes nearly dormant** for viewed folders, since `views`
  is never pruned; the 1 GB size cap (least-recently-opened first) and
  Clear Cache remain the reclaim paths. Accepted by the user; `usage.md`
  says so plainly.
- **Migration drops columns with `ALTER TABLE ... DROP COLUMN`**, which
  rewrites only `folders`; fresh and migrated databases keep one shape.
- **Between Step 1 and Step 2 nothing auto-refits** (only `Refit Ranking`),
  and between Step 2 and Step 3 a view is recorded on first draw with no
  dwell, as on `main` today. Both are short-lived intermediate states.

## Progress

- (2026-10-14) Step 1 complete
- (2026-10-14) Step 2 complete
- (2026-10-14) Step 3 complete
