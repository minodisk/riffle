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

# Retry a failed sidecar write with a bounded backoff

## Purpose

`crates/app/src/sidecar.rs`'s writer thread reports a failed sidecar write
through `on_error` (which `main.rs` turns into the `sidecar-error` event the
status line shows) and then forgets the entry: the `ratings` row stays
`dirty`, and the judgement only reaches the sidecar when the folder is opened
again, because `scan_folder` replays dirty rows with `set_now`. A transient
failure (a card that was briefly busy, a share that reconnects, a directory
that was read-only for a moment) therefore costs the user a folder reopen for
every affected file, and an error they may not have noticed. This closes the
`todo.md` item "App: a failed sidecar write is never retried until the folder
is reopened": a failed write is retried by the writer itself, a bounded number
of times with a growing delay, and the folder-reopen replay stays as the
backstop once the retries are exhausted.

Removing the resolved `todo.md` heading happens in wrap-up, not in the step.

## Steps

- [x] Step 1: Requeue a failed write in the sidecar writer with a bounded backoff
  - Done when:
    - A write that fails in `flush` (`crates/app/src/sidecar.rs`) is put back
      into `pending` with a later deadline instead of being dropped, and is
      attempted again when that deadline passes, without any call from the
      frontend or a folder reopen.
    - The retry is bounded: a per-entry attempt counter and a pure
      `fn retry_delay(attempts: u32) -> Option<Duration>` (or equivalent) that
      returns a growing delay (e.g. 1 s, 2 s, 4 s, 8 s, 16 s) and `None` once
      the cap is reached; on `None` the entry is dropped exactly as today and
      the dirty row is left for the next folder open. The constants and the
      reasoning are documented next to `DEBOUNCE`.
    - Coalescing and latest-state-wins are preserved: a new `Message::Set`
      for a path that is waiting for a retry replaces the queued judgement
      (and resets the attempt counter, since it is a new judgement) — the
      existing `pending.insert` already does the replacement; the new struct
      must not break it.
    - A drain (`flush` with `now: None`, i.e. the quit-time `Flush` and the
      channel disconnect) never requeues: it makes one attempt per entry,
      reports a failure, and lets the row stay dirty, so quitting cannot loop
      or wait out a backoff. `DRAIN_TIMEOUT` semantics are unchanged.
    - `on_error` is still called on every failed attempt, so the
      `sidecar-error` event and the status line keep working unchanged; the
      message may say that a retry is scheduled.
    - Tests in `sidecar.rs`:
      - unit test of `retry_delay`: grows, and is `None` after the cap;
      - `#[cfg(unix)]` failure-then-success: lock the directory (`0o500`, as
        `a_failed_write_leaves_the_row_dirty` does), `set` a rating, wait
        until `on_error` has fired at least once (count it through an
        `Arc<AtomicUsize>` in the closure passed to `Writer::spawn`), restore
        `0o700`, then `eventually` the sidecar exists with that rating and
        `dirty_rows` is empty — with no `flush` and no second `set`;
      - `#[cfg(unix)]` latest-state-wins during the backoff: after the first
        failure, `set` a different rating for the same path, unlock, and
        `eventually` the sidecar holds the newer rating and the row is clean;
      - the existing `a_failed_write_leaves_the_row_dirty` (drain path) still
        passes and returns promptly, i.e. the drain did not wait for a retry.
    - The stale doc comments are updated: `Writer::spawn` ("the row stays
      dirty and is retried on the next folder open"), `SidecarError` in
      `crates/app/src/main.rs`, the comment above the `sidecar-error` listener
      in `crates/app/ui/src/main.ts`, and the README sentence "A judgement
      that could not be written (say, on a locked card) is kept and retried
      the next time the folder is opened" (`README.md` ~line 166), which now
      mentions the in-session retries before the folder-open backstop.
    - `mise run ci` passes.
  - Implementation approach:
    - Replace the `Pending` value tuple `(Judgement, SidecarFormat, Instant)`
      with a small struct (e.g. `Entry { judgement, format, deadline,
      attempts: u32 }`); `run`'s min-deadline wait and `flush`'s due filter
      then work unchanged for retry entries, because a retry is just an entry
      whose deadline is in the future.
    - In `flush`'s `Err(e)` arm: `on_error(&path, &e)`, then if `now.is_some()`
      and `retry_delay(entry.attempts + 1)` is `Some(delay)`, reinsert the
      entry with `deadline = Instant::now() + delay` and the incremented
      counter. The `mark_written` `Err` (an index failure, not a disk one)
      can go through the same arm for simplicity or keep the report-only
      behaviour; decide in the step and note it in `learnings.md`.
    - Keep the backoff base at ~1 s so the failure-then-success test
      completes well inside `eventually`'s 10 s budget; do not add a
      configurable delay to `Writer::spawn` just for tests (the pure
      `retry_delay` unit test covers the cap without waiting it out).
    - The frontend needs no code change: the listener already drops payloads
      for other folders and the status note is transient. Do not touch the
      "sticky error display" item in `todo.md` (line ~151).
    - `docs/agents/tauri-app.md` needs no change unless a new pitfall is hit.

## Trade-offs and risks

- **Retry policy: timer-based backoff (chosen) vs "retry on the next
  `set_rating` for that path".** Retrying only on the next keypress for the
  same file helps nobody who has already moved on to the next photo; a backoff
  inside the writer thread is self-contained and keeps the coalescing map as
  the single queue.
- **Bound.** Five attempts over ~31 s covers a card reconnect or a transient
  lock, and a genuinely read-only share stops generating events in under a
  minute. The folder-open replay remains as the unbounded backstop.
- **Error reporting frequency.** Every failed attempt is reported (no
  contract change for `main.rs` / the frontend; approved by the user).
- **Drain path.** The quit-time drain is single-attempt, so a failure right at
  quit still needs the folder reopen — unchanged from today.
- **Timing-sensitive tests.** The failure-then-success test depends on real
  time (first retry after ~1 s). If it flakes on CI, raise the `eventually`
  budget rather than shrink the delay.

## Progress

- (none yet)
