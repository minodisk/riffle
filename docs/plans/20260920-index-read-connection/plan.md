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

# Separate read connection for the index

## Purpose

`folder_entries` and `thumbnail` (`crates/app/src/commands.rs`) take the same
`Mutex<Index>` that `run_scan`'s batch writer and the sidecar writer thread
hold, so filmstrip and meta-pane reads stall behind every `write_batch`
transaction during a folder scan. #56 only shortened the critical section by
dropping `BATCH` from 50 to 10, at an unmeasured throughput cost (5x more
transactions). The database is already in WAL mode, where a second connection
reads a consistent snapshot without waiting for the writer. Giving the
read-only commands their own connection removes the serialisation, and
measuring the per-transaction cost lets `BATCH` be chosen on evidence rather
than as a contention workaround. Closes the `todo.md` item "App: read-only
index commands share one `Mutex<Index>` with the scan writer" (the heading is
removed at wrap-up, not in the step).

## Steps

- [x] Step 1: Open a dedicated read connection for `folder_entries` / `thumbnail`, prove it never waits on the writer, and measure the `BATCH` cost
  - Done when:
    - `crates/app/src/index.rs` can open a second, read-only `Index` on the same database file (e.g. `Index::open_reader(path)`), opened with `OpenFlags::SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX` (no `SQLITE_OPEN_CREATE`), a `busy_timeout` set, and without running `prepare` (the writer already owns the schema and the WAL/synchronous pragmas; `journal_mode` is persistent in the file).
    - `crates/app/src/main.rs` opens the reader right after `Index::open` succeeds and manages it as its own state (e.g. `commands::AppIndexReader(Option<Arc<Mutex<Index>>>)`); if opening the reader fails it logs and falls back to the writer `Arc` so behaviour is no worse than today.
    - `folder_entries` and `thumbnail` in `crates/app/src/commands.rs` lock only the reader state; no read-only command takes the writer's `Mutex<Index>`. `scan_folder`, `start_scan`, `set_rating`, `switch_sidecar_format` and the sidecar writer still use the writer connection unchanged.
    - A test in `index.rs` opens a writer and a reader on one temp database, writes a batch, then holds the writer's mutex **and** an open write transaction (`BEGIN IMMEDIATE` on `writer.conn`) while a second thread calls `reader.entries(..)` and `reader.thumbnail(..)`; both return the committed rows well within a bounded wait (assert the elapsed time is far under the writer's hold, and the thread joins). A second assertion checks the reader sees rows committed after it was opened.
    - A `#[ignore]`d timing test in `index.rs` writes N synthetic rows (N >= 2000, the existing `entry()` fixture) through `write_batch` in chunks of 10 and of 50 (and optionally 100), reports ms per transaction and total ms per chunk size, and the numbers are recorded in `learnings.md` together with the machine they were taken on. The cost of `BATCH = 10` versus `50` is then stated as a fraction of the README's 5000-file first-scan baseline (5.55s).
    - `BATCH` is set to the value the measurement justifies, and the doc comment on `BATCH` is rewritten to reflect the new situation (reads no longer contend; the remaining reasons are quit-loss granularity, `set_rating` / the sidecar writer sharing the writer mutex, and the measured per-transaction cost).
    - `cancelling_after_the_first_batch_keeps_what_was_written` still passes; if `BATCH` changes, its `BATCH * 40` fixture count is revisited so the test does not balloon (see risks).
    - `docs/agents/tauri-app.md` gains a short "Measured" entry only if the measurement produced a non-obvious fact; otherwise leave it.
    - `mise run ci` passes.
  - Implementation approach:
    - Keep `Index` as the single type; the reader is just another `Index` whose caller only invokes `entries` / `thumbnail`. Do not introduce a separate `IndexReader` type or move the read methods; that duplicates code for no gain.
    - `Index::open_reader` must be called only after `Index::open` has returned `Ok` for the same path (the `-wal`/`-shm` files and the schema exist by then); say so in its doc comment. `Index::open`'s "discard and rebuild" path already runs before any reader exists, so the reader never sees the rebuilt file mid-flight.
    - Set `busy_timeout` (a few hundred ms) on the reader: WAL readers do not block on writers, but SQLite can still return `SQLITE_BUSY` transiently (e.g. during WAL recovery or a checkpoint at the wrong moment), and today's code has no retry path.
    - In `commands.rs`, mirror the existing `let Some(index) = app.state::<AppIndex>().0.clone() else { .. }` shape for the new state; keep the `spawn_blocking` wrappers as they are.
    - For the concurrency test, hold the writer lock via `index::lock(&writer_mutex)` and run `conn.execute_batch("BEGIN IMMEDIATE")` inside it, sleep e.g. 300 ms, `ROLLBACK`, then join. Measure elapsed on the reader thread and assert it is under, say, 100 ms.
    - For the timing test, measure `write_batch` directly rather than `run_scan`: extraction of the synthetic fixtures is trivial, so a `run_scan` timing would only measure the database anyway, while `write_batch` per chunk size gives the transferable number (ms per commit). Mark it `#[ignore]` and run it with `cargo test -p riffle-app --release -- --ignored` (name to be chosen); note in `learnings.md` that a debug build overstates it.
    - `learnings.md`: record the numbers, the machine, and the `BATCH` decision with its arithmetic.

## Decisions (approved by the user)

- One reader connection, not a pool.
- `BATCH`: keep 10 if the 50→10 change costs under ~2% of the 5.55s 5000-file first-scan baseline; otherwise raise it back to 50 and change `cancelling_after_the_first_batch_keeps_what_was_written` to a fixed file count (e.g. 400) rather than `BATCH * 40`.
- If a CI platform rejects `SQLITE_OPEN_READ_ONLY`, fall back to `SQLITE_OPEN_READ_WRITE` without `CREATE` and record which was needed.

## Trade-offs and risks

- **Single reader connection.** One reader behind its own `Mutex` still serialises `folder_entries` (a 5000-row join) with the up-to-4 in-flight `thumbnail` calls. A pool is a follow-up todo if that turns out to stall.
- **Synthetic-file measurement.** The timing test measures commit cost, not the real scan, where extraction (~12-17 ms per file per worker) dominates; the fraction reported against the README baseline is an upper bound.
- **Sidecar writer contention remains.** The sidecar writer thread still shares the writer mutex with `run_scan`; that affects `set_rating` latency during a scan, not reads, and is out of scope.

## Progress

- Step 1 done: added a dedicated read-only reader connection for
  `folder_entries` / `thumbnail`, a concurrency test proving reads never wait
  on the writer, and a timing test measuring `write_batch` cost. `BATCH`
  stays 10 (see `learnings.md` for the measured numbers and arithmetic).
