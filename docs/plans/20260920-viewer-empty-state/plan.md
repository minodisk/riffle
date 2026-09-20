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

# Empty state for the viewer

## Purpose

On first launch (and whenever no folder is open) the viewer is a blank canvas;
the only hint is a hardcoded grey note in the meta pane ("Press "o" or click
"Open folder"."). A folder that is open but shows nothing (no ARW/DNG files,
or everything filtered out) looks the same as an unopened one. This work puts
a visible, clickable empty state in the centre of the viewer for the
"no folder" case, wired to the existing folder picker and showing the real
`open` shortcut from the keymap, and a distinct, non-clickable message for the
"folder open but nothing to show" case.

Current state (see `crates/app/ui/src/main.ts`):

- "Empty view" is a state, not an element: `files.length === 0` handled in
  `refilter()` (~line 459) and `openDirectory()` (~line 1069). `draw()` only
  clears the canvas when `shown === null`.
- The three states are derivable from existing variables: `openDir === null`
  (no folder), `allFiles.length === 0` (folder has no RAW files),
  `files.length === 0` with `allFiles.length > 0` (filtered out).
- `openFolder()` (~line 1078) is the single picker path, already used by the
  `open-folder` menu event, the `#open` button and the `"open"` keymap action.
- The keymap arrives asynchronously via the `shortcuts` invoke and the
  `shortcuts-changed` event, both through `applyKeymap(bindings: Binding[])`
  (~line 1437). Default `open` key is `o` (`crates/app/src/shortcuts.rs`).
- Tests run under vitest with `environment: node` (root `vite.config.ts`), so
  anything unit-tested must be free of DOM access.

Decisions already taken (do not revisit):

- The overlay goes inside a new `#viewer` wrapper around the canvas, absolutely
  positioned; the layout must not hardcode the `#side` / `#meta` pane widths.
- The hardcoded meta-pane hint (`note` initial value) is removed, so the hint
  lives in one place and reflects the real keymap.
- The two zero-file causes get two distinct sentences, both distinct from the
  "no folder" one.
- Before the keymap resolves, the hint omits the key clause rather than showing
  a placeholder.

## Steps

- [x] Step 1: Pure empty-state logic with unit tests
  - Done when: `crates/app/ui/src/empty.ts` exports (names indicative)
    `emptyState(openDir: string | null, total: number, shown: number)` returning
    `"none" | "no-folder" | "no-files" | "filtered"`, and
    `openHint(bindings: Binding[])` (plus a small `displayKey`) returning the
    "no folder" sentence(s) that embed the `open` action's actual keys (and
    omit the key clause when the action has no keys or the keymap has not
    resolved yet); `crates/app/ui/src/empty.test.ts` covers the four states,
    the default `o` key, multiple keys, a rebound key (e.g. `ctrl+o`, `space`),
    and the no-key case; `mise run ci` passes.
  - Implementation approach:
    - Follow `keys.test.ts` style (`describe`/`test`/`expect` from vitest,
      `.js` import suffix). No DOM in this module.
    - Import `Binding` from `./keys.js`; do not change `settings.ts`.
    - Text is English. "No folder" text must name both ways in: dropping a
      folder onto the window and clicking (this area) to choose one, plus the
      key. The "no files" and "filtered" texts are two distinct sentences (for
      example "This folder has no ARW or DNG files." and "No files match the
      current filter."), both different from the "no folder" one.
    - `displayKey` is a minimal formatter in `empty.ts` (`space` -> `Space`,
      otherwise as-is); leave the one-off formatter in `settings.ts` untouched.
    - The hint string is what the DOM will show, so the test pins the exact
      wording; keep the sentence composition in one place.

- [ ] Step 2: Overlay element, click-to-open, keymap-driven re-render
  - Done when: with no folder open the message from step 1 is centred over the
    viewer and clicking it opens the native picker (same `openFolder()` as
    menu/key/button); opening any folder (picker, drop, `last_folder` reopen)
    hides it; a folder with no ARW/DNG files, or a filter that excludes every
    file, shows the corresponding other message and clicking it does nothing;
    rebinding the `open` key in the settings window updates the shown key
    without restart; the hardcoded meta-pane hint is gone; `mise run ci`
    passes; verified by running `mise run tauri:dev`.
  - Implementation approach:
    - Assumes step 1 is merged.
    - `index.html`: wrap `<canvas id="canvas">` and a new `<div id="empty"
      hidden>` in `<div id="viewer">`; move `flex: 1; min-width: 0;
      min-height: 0` from `#canvas` to `#viewer` and give it
      `position: relative`; canvas becomes `width: 100%; height: 100%`
      (`draw()` and `drawZoom()` read `canvas.clientWidth/Height`, so the
      canvas must still fill the viewer; check this by resizing the window and
      with the 1:1 zoom). `#empty` is absolutely positioned, centred with flex,
      `color: #999` like `#meta .note`, `text-align: center`, and gets
      `cursor: pointer` only in the clickable state (e.g.
      `#empty[data-state="no-folder"]`).
    - `main.ts`: a `renderEmpty()` (or a block inside `renderMeta()`, which
      already runs on every state change: `setStatus`, `refilter`, `show`,
      error/scan updates) computes `emptyState(openDir, allFiles.length,
      files.length)`, toggles `hidden`, sets `data-state` and `textContent`.
      Keep the current keymap bindings (the `Binding[]` handed to
      `applyKeymap`) in a module variable so `openHint` can read them, and
      call the render from `applyKeymap` so the key appears once `shortcuts`
      resolves and updates on `shortcuts-changed`.
    - Click: one listener on `#empty` that calls `openFolder()` only when the
      state is `"no-folder"` (do not add a second invoke path; do not attach a
      listener to the canvas). Use plain `textContent`, not `innerHTML`.
    - Retire the hardcoded `note` initial value (set to `undefined`) and reword
      the `reopenLastFolder` catch comment accordingly. Decide, and note in
      `learnings.md`, whether the `"No RAW (ARW/DNG) files in that folder."`
      meta-pane note stays now that the overlay says the same thing.
    - `body.dragging` outline stays as is; the empty state is not a drop
      target of its own (Tauri delivers drops via `tauri://drag-drop` on the
      whole window, see `docs/agents/tauri-app.md`).
    - Verify manually: first launch with no remembered folder; reopen with a
      remembered folder (no flash of the hint is acceptable but check it
      disappears); open an empty folder; apply a filter that hides everything
      and reset it; rebind `open` in Settings.

- [ ] Step 3: Document the empty state in the README
  - Done when: the "Features" paragraph in `README.md` (around line 41,
    "Open a folder from the picker, or drop...") mentions that with no folder
    open the viewer shows a prompt that can be clicked to open the picker;
    `mise run ci` passes.
  - Implementation approach:
    - One or two sentences, user-facing only (README content policy: no
      implementation detail, no verification log).

## Trade-offs and risks

1. `renderMeta()` is called often (every judgement, page turn, scan tick).
   Setting `textContent` on a hidden element each time is cheap, but keep the
   render idempotent and avoid re-creating nodes.
2. Moving `flex: 1; min-width: 0; min-height: 0` onto the new `#viewer` wrapper
   changes how the canvas gets its size. If `canvas.clientWidth/Height` stop
   matching the viewer, `draw()` and `drawZoom()` misrender; verify by resizing
   the window and in 1:1 zoom.
3. Sharing one key formatter between `empty.ts` and `settings.ts` via `keys.ts`
   is a small refactor outside this request; not done here.

## Progress

- (none yet)
