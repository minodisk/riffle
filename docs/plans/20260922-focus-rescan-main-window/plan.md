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

# Scope the focus rescan to the main window

## Purpose

`crates/app/ui/src/main.ts` subscribes `tauri://focus` through the global
`window.__TAURI__.event.listen`, which the Tauri JS bundle registers with
`target: { kind: "Any" }`. Tauri 2.11's `emit_js_filter` delivers a
window-scoped event to every webview and `match_any_or_filter`
(`tauri-2.11.6/src/event/listener.rs:306-311`) lets an `Any` listener bypass
the per-window filter, so the main webview's listener fires when the
**settings** window gains focus too. When the native Clear Cache dialog
(parented to `settings`) closes, the settings window regains focus, the main
window runs `resync()` -> `scan_folder`, and `clear_index`'s second
`scanning()` check (`crates/app/src/commands.rs:1480`) refuses with
`a scan is running; wait for it to finish`. When the race goes the other way
the clear succeeds, `index-cleared` calls `openDirectory`, and its
`scan_folder` (N+1) cancels the focus rescan's scan N before it reaches
`scan extract`, which is the "scan started twice" observation.

Making the focus listener fire only for the main window's own focus fixes both
todo.md items with one change; no Rust change is needed.

## Steps

- [x] Step 1: Listen to `tauri://focus` on the main window only
  - Done when:
    - `main.ts` registers the focus rescan with a window-scoped target, so
      the settings window's focus no longer reaches it.
    - `pnpm exec vp check` (type check) passes with the extended
      `tauri.d.ts`.
    - `mise run ci` passes.
    - Manual check on Windows (`mise run tauri:release:devtools`, timing
      logs on), done by the user after merge: open a folder, wait for
      `scan-done`, open Settings > Cache, press Clear Cache, then Clear.
      Expect: no refusal, `#status` shows `Clearing the index cache…`, the
      size drops, the main window rescans, and the log shows exactly one
      `scan list` / `scan extract` pair for the reopen (no `scan_id` N
      superseded by N+1). Then click back to the main window: one focus
      rescan fires (one `scan list` line), not two.
    - `todo.md`: remove the item "App: Clear Cache is refused on Windows
      with no scan running" and the item "App: a scan can be started twice
      after a cache clear / focus rescan". Update the "Clear Cache button's
      manual GUI verification" item so it no longer says checks (2)-(7) are
      blocked by the refusal bug (leave its checklist open).
    - `docs/agents/tauri-app.md`: add a short "Hit" item explaining that a
      global `event.listen` for `tauri://focus` receives every window's
      focus, with the fix.
  - Implementation approach:
    - Replace `void window.__TAURI__.event.listen("tauri://focus", resync);`
      (`crates/app/ui/src/main.ts:1461`) with
      `void window.__TAURI__.window.getCurrentWindow().listen("tauri://focus", resync);`.
      The bundle's `Window.listen` registers with
      `{ target: { kind: "Window", label: this.label } }`, which is what
      `emit_to_window`'s filter matches. Both go through the same
      `plugin:event|listen` command already allowed by `core:event:default`,
      so no capability change.
    - Extend the hand-written `crates/app/ui/src/tauri.d.ts`: add
      `listen<T>(event: string, handler: (event: TauriEvent<T>) => void): Promise<() => void>;`
      to the `getCurrentWindow()` return type.
    - Update the comment above the listener to say the listener is
      window-scoped and why.
    - Do not touch `clear_index` in `commands.rs`: its two `scanning()`
      checks are correct; the spurious scan was the bug.
    - Regression test: no automated test is feasible (`main.ts` has no unit
      tests and the cause is Tauri's event routing). The manual check above
      is the verification; record it in `learnings.md`.

## Trade-offs and risks

- If the manual check on Windows still shows two `scan list` lines per
  single focus, reopen the "started twice" todo item with that log.
- `resync` on the main window's own focus still fires after the user clicks
  back from Settings; that is the intended focus rescan and unchanged.

## Progress

- Step 1: done (focus listener scoped via getCurrentWindow().listen; manual Windows check pending, run by the user after merge)
