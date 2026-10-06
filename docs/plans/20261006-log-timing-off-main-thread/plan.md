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

# Run `log_timing` off the main thread

## Purpose

`log_timing` in `crates/app/src/main.rs` is a synchronous `#[tauri::command]`,
and a non-`async` Tauri 2 command runs inline on the main thread (see
"Synchronous commands run on the main thread" in `../../agents/tauri-app.md`).
The frontend's `debugLog` in `crates/app/ui/src/main.ts` invokes it
fire-and-forget on hot paths (page flips, zoom, refresh entries during scans),
so every timing line is currently appended to `Riffle.log` by
`tauri_plugin_log` on the main thread. Making the command `async` moves the
write onto the async runtime, so the timing logs stop costing the run loop the
very latency they are there to measure.

## Steps

- [x] Step 1: Make `log_timing` an `async fn`
  - Done when:
    - `log_timing` in `crates/app/src/main.rs` is declared `async fn` and is
      still listed in the `generate_handler!` invoke handler (line ~820)
    - Its doc comment still says what it does and adds one sentence on why it
      is `async` (a sync command would write the log line on the main thread,
      and the frontend calls it on hot paths)
    - `timing_logs`, `set_timing_logs`, the plugin setup and the frontend call
      site (`debugLog`, `crates/app/ui/src/main.ts` line ~147) are untouched
    - `mise run ci` passes
  - Implementation approach:
    - Change only the signature (`fn log_timing` -> `async fn log_timing`) and
      the doc comment; the body stays `log::info!("{line}")`. No return type is
      needed; `set_last_viewed` in `commands.rs` is an existing `async`
      command with no return value, so this matches the codebase
    - Do not add `spawn_blocking` (see Trade-offs; the user chose the plain
      `async fn`)
    - Commit as `perf(app): run log_timing off the main thread`

## Trade-offs and risks

- `async fn` versus `async fn` + `spawn_blocking`: `tauri-app.md` says an
  `async` command doing blocking IO should wrap it in
  `tauri::async_runtime::spawn_blocking`, since a blocking call inside the
  future stalls an async runtime worker. A log append through
  `tauri_plugin_log` is a tiny, short file write, and the constraint asks for
  the minimal change, so the plan takes the plain `async fn` (confirmed by the
  user). Wrapping the `log::info!` in `spawn_blocking` would be a two-line
  addition, at the cost of a thread hop per log line on a path that is already
  fire-and-forget.
- `#[tauri::command(async)]` on a non-`async` fn is not an alternative: per
  the guide, it runs the fn inside the future on a runtime worker, which is
  the same as `async fn` but less explicit. Use `async fn`.
- Behavior: log ordering across lines stays the same in practice (the plugin
  serializes writes), but lines are no longer guaranteed to be written before
  the next IPC call from the frontend returns. Nothing reads `Riffle.log`
  synchronously, so this is not a regression.

## Progress

- (none yet)
