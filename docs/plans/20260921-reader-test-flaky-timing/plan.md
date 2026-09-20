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

# Make the open-write-transaction reader test robust to scheduling noise

## Purpose

`index::tests::the_reader_does_not_wait_on_an_open_write_transaction` in
`crates/app/src/index.rs` asserts that two reads taken while a write
transaction is open complete in under 100 ms of wall-clock time. That bound
measures thread spawn, mutex acquisition and two queries under whatever load
the machine happens to be under, and it failed once at 161 ms during an
unrelated `mise run ci`. The property the test exists for (a WAL reader is
not blocked by an open `BEGIN IMMEDIATE` transaction) can be shown without a
timer, so the test should assert that instead.

## Steps

- [x] Step 1: Replace the wall-clock assertion with a structural one and remove the todo entry
  - Done when:
    - The test no longer asserts any elapsed-time bound.
    - The test still fails if a reader were blocked by the open write
      transaction, and still checks the follow-up write is visible to the
      reader (the existing `4` entries assertion).
    - The `### App: a Rust test is flaky under load — ...` section and its
      `#### TODO` block are removed from `todo.md`.
    - `mise run ci` passes.
  - Implementation approach:
    - In `crates/app/src/index.rs` (test around lines 1919–1954): keep
      `BEGIN IMMEDIATE` on the writer, spawn the reader thread as now, but
      `join()` it (and unwrap the `(entries.len(), thumb.0) == (3, 6)`
      result) **before** executing `ROLLBACK`. Drop the `Instant`/elapsed
      tuple element, the `hold` sleep and the `< 100ms` assertion.
    - Why this proves the property: the reader connection is opened by
      `Index::open_reader`, which sets `busy_timeout(300 ms)`. If the reads
      were blocked by the writer they would return `SQLITE_BUSY` after the
      timeout (the writer cannot roll back until the join returns), so the
      reader's `unwrap()` panics and `join().unwrap()` fails. Success can
      only happen while the transaction is still open. Add a short comment
      in the test saying this, so the dependency on `busy_timeout` in
      `open_reader` is visible to whoever edits either.
    - Check for `Instant`/`Duration` imports that become unused in the test
      module and remove only those made unused by this change.
    - Do not change `open_reader`, `busy_timeout`, or other tests.
    - Run the test in a loop (e.g. `cargo test -p riffle-app
      the_reader_does_not_wait -- --test-threads=1` several times, or while
      `mise run ci` runs in parallel) to confirm it stays green under load.

## Trade-offs and risks

- **Structural assertion (chosen) vs. loosening the bound**: raising the bound
  to e.g. 1 s would still be a timing assertion and would still be under the
  300 ms `busy_timeout`'s shadow only by accident; any bound is either too
  tight under load or too loose to distinguish "not blocked" from "blocked
  for a while". Joining before rollback makes the test deterministic.
- **Dependency on `busy_timeout`**: if `open_reader` ever stops setting a busy
  timeout, a genuinely blocked reader would make this test hang (writer waits
  on join, reader waits on writer) instead of failing. The comment in the
  test mitigates this; alternatively the join can be wrapped in a generous
  watchdog, but that reintroduces a timer for no expected gain.

## Progress

- (none yet)
