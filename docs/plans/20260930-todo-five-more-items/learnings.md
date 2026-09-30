# Learnings

## Step 1: the flaky switch-to-Both sidecar test

- Root cause analysis. The test queues the dirty row with `set_now` (deadline
  `Instant::now()`) and then sends `Flush`. The writer thread handles the
  `Set` first, then either its `recv_timeout(0)` fires and it writes the row
  (`flush(Some(now))`) or it receives the `Flush` and writes it
  (`flush(None)`); either way the reply to `Flush` is sent only after the
  write, so message order is not a race. The read can only see no file when
  (a) `Writer::flush`'s `recv_timeout(DRAIN_TIMEOUT)` gave up after 2 s while
  the writer was still inside `write_kind` for two files (`File::create` +
  `sync_all` + `rename`, and the Windows `rename` retries up to 5 times with
  20-100 ms sleeps on a sharing violation) on a loaded `windows-latest`
  runner, or (b) the write failed, whose message only went to the test's
  `on_error`, which printed it to stderr and forgot it. Both were silent, so
  the failure surfaced one step later as `NotFound` on the read. Which of the
  two happened in the failed run cannot be told from its log; the fix makes
  either one fail loudly at the drain.
- The fix: `Writer::flush` now returns `bool` (`true` when the drain reply
  arrived in time). Production callers keep ignoring it; `bool` is not
  `#[must_use]`, so clippy stays quiet. The test's `switch_writer` now
  collects every `on_error` message into an `Arc<Mutex<Vec<String>>>`, and a
  new `drain` helper flushes with a 30 s budget and asserts both the flush
  result and an empty error list before any sidecar is read.
- The two sibling tests in the same shape
  (`a_rating_set_during_a_switch_lands_in_the_new_format_and_leaves_no_dirty_row`
  and `a_pick_set_during_a_switch_lands_in_the_dop_and_leaves_no_dirty_row`)
  were changed too: `switch_writer` was used only by these three tests, so
  they all go through the collecting writer and `drain` now.
- Side effect to be aware of: a write that fails once on the deadline path
  and then succeeds on its retry or on the drain still leaves its
  `(retrying in Ns)` message in the collected list, so the test fails on a
  transient error it would have survived before. That is intended here (a
  failing sidecar write in these tests is itself worth seeing), but if it
  ever shows up on CI as a sharing violation, the message now says so.
