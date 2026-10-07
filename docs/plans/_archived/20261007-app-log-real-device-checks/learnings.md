# Learnings

## Step 1: evidence of the real-device checks (Windows, debug build, 2026-10-07)

These are the results that closed the `todo.md` item "### App: real-device
checks for the agent-readable app log (panic, uncaught-js and invariant
lines)". They are kept here so they survive the archive move.

- Normal session: a cold folder scan, a focus rescan after 5 s, a folder
  switch mid-scan that canceled `scan_id=10` at 282/936, and Clear Cache
  evicting 48984 rows. `Riffle.log` had no `invariant:` line and no WARN /
  ERROR.
- DevTools `setTimeout(() => { throw new Error("x") })` logged:

  ```text
  uncaught-js: kind=error message=Uncaught Error: x line=1 col=26 stack=Error: x | at <anonymous>:1:26
  ```

- DevTools `Promise.reject(new Error("y"))` logged:

  ```text
  uncaught-js: kind=unhandledrejection message=y stack=Error: y | at <anonymous>:2:16
  ```

- A scratch `panic!("test")` at the top of `run_scan` logged the line below,
  plus the default stderr panic message. The scratch change was reverted, so
  the location does not match the committed source.

  ```text
  panic: thread=tokio-rt-worker location=crates\app\src\index.rs:1559:5 payload=test
  ```

## Step 1: notes

- The plan cited lines 1437–1448 at `d165723e`; on this branch the section
  sat at lines 1484–1495 (the file had grown), so it was located by its
  heading rather than by line number. The removed span was still the blank
  separator plus the 12 section lines, and the file now ends with
  `- [ ] If it should, skip them and count them in the dialog's "skipped" figure.`
  and a single newline.
- A grep of `docs/agents/` and `docs/humans/` for `uncaught-js` /
  `invariant:` found no sentence describing these checks as unrun.
