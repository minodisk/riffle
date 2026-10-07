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

# Close the agent-readable app log real-device checks

## Purpose

The `todo.md` item "### App: real-device checks for the agent-readable app
log (panic, uncaught-js and invariant lines)" (the last section of the file,
lines 1437–1448 at `d165723e`) asked for a hands-on run of the hooks the
agent-readable-app-log feature added
(`docs/plans/_archived/20261007-agent-readable-app-log/plan.md`): the
uncaught JavaScript error / rejection forwarding, a real Rust panic on a scan
worker, and the "no `invariant:` line in a normal session" criterion. All
three checks ran on Windows on 2026-10-07 in a debug build and passed:

- Normal session: a cold folder scan, a focus rescan after 5 s, a folder
  switch mid-scan that canceled `scan_id=10` at 282/936, and Clear Cache
  evicting 48984 rows. `Riffle.log` had no `invariant:` line and no WARN /
  ERROR.
- DevTools `setTimeout(() => { throw new Error("x") })` and
  `Promise.reject(new Error("y"))` logged
  `uncaught-js: kind=error message=Uncaught Error: x line=1 col=26 stack=Error: x | at <anonymous>:1:26`
  and
  `uncaught-js: kind=unhandledrejection message=y stack=Error: y | at <anonymous>:2:16`.
- A scratch `panic!("test")` at the top of `run_scan` logged
  `panic: thread=tokio-rt-worker location=crates\app\src\index.rs:1559:5 payload=test`
  plus the default stderr message (scratch change reverted).

The repo convention is to delete a verified check item (for example
`docs/plans/_archived/20261004-follow-up-rescan-log-check/plan.md` and
`docs/plans/_archived/20261005-macos-view-menu-checks/plan.md`), so the
evidence lives here, in the archived plan, and the item goes.

## Steps

- [x] Step 1: Delete the verified item from `todo.md`
  - Done when:
    - `todo.md` no longer contains the section "### App: real-device checks
      for the agent-readable app log (panic, uncaught-js and invariant
      lines)": the heading, its `#### Background` paragraph, the `Files:`
      line, the `#### TODO` heading and both `- [ ]` items (lines 1437–1448
      at `d165723e`), together with the blank line 1436 that separated it
      from the previous section.
    - No other line of `todo.md` changes. Because the item was the last
      section, the file now ends with the previous section's last line
      (`- [ ] If it should, skip them and count them in the dialog's
      "skipped" figure.`) followed by a single newline.
    - The three check results above are recorded in this plan folder's
      `learnings.md` (the log lines verbatim), so the evidence survives the
      archive move.
    - No code changes. `mise run ci` passes.
  - Implementation approach:
    - Docs only. Files: `todo.md` and this plan folder (`plan.md`,
      `learnings.md`).
    - Commit as `docs(todo): close the agent-readable app log real-device
      checks`.

## Trade-offs and risks

- Nothing in `docs/agents/` or `docs/humans/` refers to these checks as
  unrun, so `todo.md` is the only file touched. If the implementer finds such
  a sentence while grepping for `uncaught-js` / `invariant:`, flag it rather
  than widening the PR.
- The panic location `crates\app\src\index.rs:1559:5` is from the scratch
  build and will not match the committed source; it is recorded as evidence
  of the hook's output, not as a source reference.

## Progress

- (none yet)
