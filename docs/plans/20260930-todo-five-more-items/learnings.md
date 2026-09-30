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
- Only final errors fail the test: a write that fails once on the deadline
  path (`set_now` makes `recv_timeout(0)` usually fire before the `Flush`)
  and then succeeds on the rewrite leaves a `(retrying in Ns)` message, and
  the row and both sidecars are correct. `drain` therefore filters out
  messages containing `" (retrying in "` and asserts the rest is empty, so
  the Windows-flaky test is not made flakier. The full list, retry messages
  included, goes in the assertion message so a transient sharing violation
  stays visible in the log; the timeout message carries it too. This departs
  from the plan's "the collected errors are empty" wording on purpose.

## Step 2: throttling the focus rescan

- `lastScanAt` is set in `startScan`, which `openDirectory` and `resync`
  share, so the throttle is measured from the last scan start of any
  trigger. `File > Reload Folder`, `folder-changed` and the trash / rename
  paths call `resync()` directly and are not throttled. A focus inside the
  interval is dropped, not deferred; a focus during a running scan that is
  past the interval still goes through `resync()`'s `resyncPending` path.
- Backend: `scan_folder`'s reconcile `spawn_blocking` now also calls
  `Index::faces_todo(&dir)` under the same index lock when the first pass's
  `todo` is empty, and only inserts the `pending` entry when either pass has
  work. The `index` handle is moved into the sidecar reconcile closure later
  on, so the check had to live in the earlier closure; doing it there also
  avoids a second lock and `spawn_blocking`. A failed `faces_todo` counts as
  "not idle", so the faces pass runs and logs the error as before.
  `faces_todo`'s side effect (marking untestable rows done) is the same as
  when `run_faces_pass` calls it first.
- No new Rust test: the "a fully scanned folder has an empty `faces_todo`"
  case is already asserted in `index.rs`'s eye-focus test (`faces_todo` is
  empty after `write_faces`), and `scan_folder` itself needs a Tauri
  `AppHandle`, which no existing test builds.
- The first try at removing the `todo.md` section cut it at the next `##`
  match, which was the section's own `#### TODO`; check the next `### `
  heading when deleting a section by line range.

## Deferred issues (todo candidates)

- **Pending manual check (Step 2, Windows, the user's):** verify the idle
  scan ends at once with `Timing logs` on. Steps: open a folder and let its
  scan and faces pass finish; wait more than 5 s; switch to another app and
  back to Riffle. Expected: `Riffle.log` shows `scan prepare: … todo=0`
  with no `scan extract` or `scan faces` line after it, and the status
  line's `scanning` never appears. Also alt-tab away and back twice within
  5 s: only the first focus logs a `scan list` line. Not run by the
  implementation agent (no GUI session); Step 2's checkbox was ticked on the
  automated criteria (Vitest cases for `focusRescanDue`, `mise run ci`).
  Basis: plan Step 2 "Tests" bullet. Files: `crates/app/ui/src/main.ts`,
  `crates/app/src/commands.rs`.
