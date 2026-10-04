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

# Retry the PhotoLab Uuid lookup on a busy database

## Purpose

`crates/app/src/photolab.rs::lookup` opens PhotoLab's database read-only with
a 300 ms `busy_timeout` and returns `None` on any error. When PhotoLab holds
the database past that window, the fresh `.dop` is written with random Uuids
and PhotoLab imports it as a virtual copy on a registered image. Retrying the
query a few times (about 2 s in total) before the sidecar is written makes
that miss rare without ever rewriting a sidecar PhotoLab may already have
read. This closes the `todo.md` section "App: a busy PhotoLab database
silently falls back to random Uuids".

## Steps

- [x] Step 1: Retry `lookup` on `SQLITE_BUSY` / `SQLITE_LOCKED`, warn on a
      final busy miss, document it, and remove the todo section
  - Done when:
    - `lookup` retries `query` when it fails with `SQLITE_BUSY` or
      `SQLITE_LOCKED` (those two codes only); every other error returns
      `None` at once and is still logged at `log::debug!`; a plain miss is
      still not logged.
    - When every attempt failed busy/locked, the failure is logged at
      `log::warn!` and `None` is returned (random Uuids, as before).
    - The per-attempt `busy_timeout` and the attempt count are `const`s in
      `photolab.rs` (e.g. `BUSY_TIMEOUT = 300 ms`, `BUSY_ATTEMPTS = 6`,
      about 1.8–2 s in total) so the tests can be read against them.
    - `sidecar.rs::write_kind` is unchanged: the lookup still runs only for a
      fresh `.dop`, and no already-written `.dop` is re-patched afterwards.
    - Two new tests in the `#[cfg(all(test, windows))]` module pass on
      Windows: (a) another connection holds an exclusive lock on the fixture
      DB and a spawned thread releases it after a delay longer than one
      attempt and shorter than the total (e.g. 2.5 × `BUSY_TIMEOUT`);
      `lookup` yields the fixture's Uuids. (b) the lock is held for the whole
      call; `lookup` yields `None`. The existing tests still pass.
    - `docs/agents/photolab.md`'s bullet "Open read-only with a short
      `busy_timeout` ..." describes the retry, the warn-level log on a final
      busy miss, and why the sidecar is not re-patched afterwards (the
      rejected alternative and its reason).
    - The `todo.md` section
      "### App: a busy PhotoLab database silently falls back to random Uuids"
      (heading, paragraph and its `#### TODO` list) is removed entirely; the
      neighbouring macOS and rename sections are untouched.
    - `mise run ci` is green.
  - Implementation approach:
    - `crates/app/src/photolab.rs`: keep `query` as is (it already opens the
      connection and sets `busy_timeout`; switch the literal 300 ms to the new
      const). Put the loop in `lookup`: call `query` up to `BUSY_ATTEMPTS`
      times; on `Err(e)` where `is_busy(&e)` holds and attempts remain,
      continue (no extra sleep is needed, each attempt already waits
      `BUSY_TIMEOUT` inside SQLite's busy handler). `is_busy` matches
      `rusqlite::Error::SqliteFailure(rusqlite::ffi::Error { code: rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked, .. }, _)`
      (rusqlite 0.40, already a dependency with `bundled`). Keep it a small
      private fn next to `lookup`.
    - Logging: the final busy/locked failure → `log::warn!` with the same
      shape as today's message plus the attempt count; any other error →
      the existing `log::debug!` message. Update `lookup`'s doc comment
      accordingly.
    - Tests: the fixture DBs are rollback-journal SQLite files, so a second
      `Connection::open(&db)` running `BEGIN EXCLUSIVE` takes the exclusive
      lock and a reader's first schema read hits the busy handler and fails
      `SQLITE_BUSY` after `BUSY_TIMEOUT`. For the success test, move the
      locking `Connection` into a `std::thread::spawn` that sleeps
      (`BUSY_TIMEOUT * 5 / 2`) then runs `COMMIT`; call `lookup` on the test
      thread and `join` the thread afterwards. For the failure test, keep the
      lock on the test thread for the whole `lookup` call and `COMMIT` after
      asserting `None`. Give each test its own fixture name. Derive every
      delay from the consts.
    - `docs/agents/photolab.md`: rewrite only that one bullet. State: read-only
      open, `busy_timeout` per attempt, retried up to N times (~2 s) on
      `SQLITE_BUSY` / `SQLITE_LOCKED` only, before the sidecar is written; a
      final busy miss logs at `log::warn!` and falls back to random Uuids;
      other failures still log at `log::debug!`; a plain miss is not logged.
      Add the rejected alternative: re-patching an already-written `.dop`
      after a later successful lookup was not done because PhotoLab may
      already have imported the sidecar as a virtual copy, and a rewrite can
      race PhotoLab's own writes to the same file.
    - `todo.md`: delete the section.
    - No manual check with real PhotoLab: a busy database cannot be provoked
      reliably by hand, so the two tests are the verification.

## Trade-offs and risks

- Re-patch after a later lookup (rejected, per the user): PhotoLab may
  already have imported the random-Uuid sidecar as a virtual copy (so the
  rewrite does not undo it) and the rewrite can race PhotoLab's own write of
  the same `.dop`. Retrying before the write is the only safe window.
- Total wait: ~2 s on the coalescing sidecar writer thread delays that
  file's write only while the database is busy.
- Test runtime: the suite grows by about 5 s (measured, see `learnings.md`).
- Retry granularity: looping around `query` (reopening per attempt) keeps the
  attempt count visible in the warn log; a single longer `busy_timeout` would
  be equivalent for SQLite but less clear to log.

## Progress

- Step 1 done: `lookup` retries a busy database up to `BUSY_ATTEMPTS` times; tests cover a lock released mid-retries (released at 2 s, past two attempts) and a lock held through every retry.
