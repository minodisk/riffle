# Learnings

## Step 1

- The fix is frontend-only: `getCurrentWindow().listen` registers with a
  `Window` target, so the settings window's `tauri://focus` no longer reaches
  the main webview's rescan. `tauri.d.ts` is hand-written, so `listen` had to
  be added to the `getCurrentWindow()` return type.
- No automated regression test is feasible (`main.ts` has no unit tests and
  the cause is Tauri's event routing). Verification is the manual Windows
  check in plan.md, run by the user after merge.
