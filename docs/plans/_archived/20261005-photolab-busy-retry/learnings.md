# Learnings

## Step 1

- On Windows, one `lookup` attempt against the rollback-journal fixture held
  by another connection's `BEGIN EXCLUSIVE` takes about 0.8 s of wall time,
  not the nominal 300 ms `busy_timeout`: SQLite's Windows `winLock` sleeps
  between its own retries of the pending lock on each busy-handler call, on
  top of the busy handler's nominal delays. So `BUSY_ATTEMPTS = 6` waits about
  4.9 s when the lock is held throughout (measured in
  `a_lock_held_through_every_retry_yields_none`), not the planned ~2 s. The
  consts were kept as planned (1.8 s of nominal busy wait) since PhotoLab's
  real database is WAL, where a busy reader goes through the shared-memory
  locks rather than that pending-lock path; the measured figure is noted in
  `docs/agents/photolab.md`.
- The test suite grows by about 5 s (the held-lock test), running in parallel
  with the other tests.
- A bare `python3` heredoc hangs in Git Bash on this host (known, see
  `todo.md`'s Agents section); edit with the Edit tool or `sed` instead.
