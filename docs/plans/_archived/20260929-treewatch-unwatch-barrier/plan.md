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

# Wait for the Windows watch handle to close before renaming

## Purpose

`treewatch::tests::release_under_releases_the_folder_and_below_and_restore_puts_them_back`
fails intermittently on the `test (windows-latest)` job with
`PermissionDenied` (os error 5) from the `std::fs::rename` right after
`release_under` (post-merge CI of #540, run 36432825515; of #551, run
36497782763; PR check of #561, run 36502702864; each passed on rerun).

The cause is in `notify` 8.2.0's Windows backend
(`~/.cargo/registry/src/*/notify-8.2.0/src/windows.rs`):
`ReadDirectoryChangesWatcher::unwatch_inner` only sends
`Action::Unwatch` on the server channel and returns without an ack
(`watch_inner` does wait, through `send_action_require_ack`; `unwatch`
does not), and `Drop` only sends `Action::Stop`. The `CancelIo` +
`CloseHandle` of the directory handle (`stop_watch`) run later on the
`notify-rs windows loop` thread. So when `unwatch()` (or dropping the
watcher) returns, the handle that pins the folder's ancestors can still be
open, and an ancestor rename issued at once races it. The earlier learning
"`unwatch` releases the handle at once (20/20 runs, ~0.3 ms)" was a lucky
measurement of that race, not a guarantee.

This is not only a test problem: `rename_folder`
(`crates/app/src/rename.rs`) renames right after `watch::release_under`
(drops the open folder's watcher) and `treewatch::with_released`
(unwatches the folder and its descendants), so renaming a folder with an
expanded subfolder in the tree, or with the open folder under it, can fail
spuriously with "Permission denied" in the app. Once done, both the tests
and the production rename wait deterministically for the handles to close.

The user chose the `configure` barrier alone (no rename retry).

## Steps

- [x] Step 1: Make the watch release synchronous on Windows and fix the rename tests and `rename_folder` through it
  - Done when:
    - `treewatch::State::remove` (and so `release_under` / `with_released`)
      returns only after the OS handle of an unwatched folder is closed.
    - `watch::release` returns only after the open folder's handle is
      closed (it currently just drops the watcher, which is asynchronous
      on Windows too).
    - No production or test code sleeps or retries around the rename;
      the two rename-after-release tests in `treewatch.rs` (lines 322 and
      355) stay as they are, exercising the same release path
      `rename_folder` uses.
    - The reproduction and the fix are measured on this Windows machine
      and recorded in `learnings.md`: the `treewatch` tests looped 200+
      times on the old code under parallel CPU load (to mimic the CI
      runner), showing at least one failure, then the same loop on the
      new code with zero failures.
    - `docs/agents/tauri-app.md`, section "A `notify` watch on Windows
      pins the watched folder's ancestors, not the folder itself (Hit)",
      is updated: `unwatch` and `Drop` are asynchronous on Windows (the
      handle closes on notify's server thread; `watch` waits for an ack,
      `unwatch` does not), the "released at once" claim is corrected, and
      the barrier used is described. `docs/plans/_archived/...` learnings
      are left as they are (history).
    - `mise run ci` passes.
  - Implementation approach (as far as it is known; omit if unknown):
    - Barrier: after `watcher.unwatch(path)`, call
      `watcher.configure(notify::Config::default())` and ignore its
      result. In notify 8.2.0 `Action::Configure` goes through the same
      action channel as `Action::Unwatch`, the server processes actions
      in FIFO order in one `while let Ok(action) = self.rx.try_recv()`
      loop, and `configure` blocks on the reply, so it returns only after
      `stop_watch` (`CancelIo`, `CloseHandle`, wait on the completion
      semaphore) ran for every preceding `Unwatch`. On inotify the call
      also round-trips through the event loop and on fsevents replies
      immediately; the release is already synchronous there, so the call
      is a harmless no-op. Put it in one small helper (e.g. a private
      `fn settle(watcher: &mut RecommendedWatcher)` in `treewatch.rs`,
      `pub(crate)` so `watch.rs` can share it) with a comment naming the
      notify internals it relies on, rather than repeating it inline. It
      may be gated with `#[cfg(windows)]` or left unconditional; prefer
      unconditional unless it measurably slows the other platforms'
      tests (it should not).
    - `treewatch::State::remove`: call the helper after the `unwatch`
      when `last` is true. This covers `apply` (stale watches),
      `release_under` and `with_released`, and both rename tests.
    - `watch::release`: instead of only `state.watcher = None`, take the
      watcher out, `unwatch(watched)`, run the helper, then drop it. The
      `Stop` sent by `Drop` is then harmless because the handle is already
      closed. Leave `watch::set`'s replacement drop alone (not a rename
      path); mention its comment "Dropping the previous watcher first
      releases the directory handle" is only eventually true in the guide
      update, do not rewrite it unless the reviewer asks.
    - `rename.rs` needs no change: it already renames inside
      `with_released` after `watch::release_under`; the barrier lands in
      the two release functions it calls. Do not add a retry loop.
    - Reproduction / verification on this machine (PowerShell or Git
      Bash): build the test binary once (`cargo test -p riffle-app
      --no-run`), then loop the two treewatch tests, e.g.
      `for i in $(seq 1 300); do cargo test -p riffle-app treewatch:: -q
      -- --test-threads=1 || echo FAIL $i; done`, while a load generator
      runs in parallel (several busy loops pinning every core, e.g. a
      `cargo build` of another target or `N` PowerShell `while($true){}`
      processes, `N` = logical CPU count). Run first on the unfixed code
      to confirm the race (record how many iterations failed), then on
      the fixed code (expect zero). If the old code will not fail even
      under load, say so in `learnings.md`; the source-level analysis
      above still stands, and the barrier is still the fix. Also run the
      whole `riffle-app` test suite a few times with default parallelism.
    - Check during implementation that `configure` does not deadlock if
      the watcher was created but the server thread already exited (it
      returns `Err` on a closed channel; ignoring the result is enough),
      and that calling it from inside the `TreeWatch` lock cannot wait on
      the notify handler (the handler only takes the `keys` lock, and
      the barrier does not take any app lock).
    - Files: `crates/app/src/treewatch.rs`, `crates/app/src/watch.rs`,
      `docs/agents/tauri-app.md`, this plan folder's `learnings.md`.

## Trade-offs and risks

- Barrier via `configure` (chosen) vs. a bounded retry of the rename on
  `PermissionDenied`: the barrier is exact and adds no sleeping, but it
  leans on notify's internal FIFO action processing, which a future notify
  upgrade could change silently (the tests would then go flaky again, so a
  notify bump should rerun the loop). A retry would be independent of
  notify internals and would also absorb other transient Windows holders
  (an antivirus scanner briefly opening the folder), but it is
  probabilistic and adds latency on real failures.
- An alternative barrier, `watch` + `unwatch` of a throwaway path (since
  `watch` waits for an ack), would also flush the queue but creates and
  closes a real handle each time; `configure` is cheaper and side-effect
  free.
- If the old code cannot be made to fail locally even under load, the
  acceptance item "show the old code fails" is not met empirically; the
  race is nonetheless proven by the notify source (no ack on unwatch), and
  the CI history is the evidence. Record this honestly in `learnings.md`.
- `watch::release` changes from "drop" to "unwatch, settle, drop". Its two
  unit tests (`release_drops_a_watch_on_the_folder_or_under_it_and_keeps_a_sibling`,
  `release_returns_the_watched_dir_and_owner`) must still pass unchanged.

## Progress

- (2026-09-29) Step 1 complete
