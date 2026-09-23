# Learnings

## Step 1

- `empty.ts` exports `emptyState`, `openHint`, `displayKey` and the two
  zero-file sentences (`NO_FILES_TEXT`, `FILTERED_TEXT`) so step 2 can set the
  overlay text without composing wording in `main.ts`.
- `openHint` treats an empty `Binding[]` (keymap not resolved) and an `open`
  action with no keys the same way: the key clause is dropped.
- `displayKey` only maps `space` -> `Space`, mirroring the one-off formatter in
  `settings.ts` (left untouched on purpose).

## Step 2

- The `"No RAW (ARW/DNG) files in that folder."` meta-pane note is **removed**.
  The overlay now says the same thing in the center of the viewer, in the same
  gray, and `note` is a transient line for errors and the scan progress; two
  copies of one sentence on screen at once only risks them drifting apart.
  `openDirectory` now just clears the status (`setStatus()`).
- `keyBindings` (the `Binding[]` last applied) is declared next to
  `renderEmpty`, above `renderMeta`, not next to `applyKeymap` near the bottom
  of the file: `renderMeta` runs during module evaluation (the last line of
  `main.ts`), so a `let` declared after it would be in its temporal dead zone.
- `#viewer` took `flex: 1; min-width: 0; min-height: 0` from `#canvas`, which
  is now `width: 100%; height: 100%`, so `canvas.clientWidth/Height` still
  match the viewer box that `draw()` and `drawZoom()` size against.
- Not verified interactively: `mise run tauri:dev` cannot be driven from this
  environment (no interactive session, no way to click the overlay or rebind a
  key in the settings window). The behavior was checked by reading the code
  paths only; a human run of the plan's manual checklist is still outstanding.
- Step 3: the README "Features" paragraph gained one sentence about the
  clickable prompt shown when no folder is open. The "no files" / "filtered"
  messages were deliberately left undocumented: they are self-explanatory in
  the app and the README stays user-facing and short.

## Deferred issues (todo candidates)

- The key formatter is duplicated between `crates/app/ui/src/empty.ts`
  (`displayKey`) and `crates/app/ui/src/settings.ts` (the local `display`);
  sharing one via `keys.ts` is out of scope here (plan "Trade-offs and risks" 3).
- Step 2 could not be verified by running the app (`mise run tauri:dev` needs
  an interactive session). The plan's manual checklist for the empty state
  (first launch, remembered folder, empty folder, filter that hides
  everything, rebinding `open`) still needs a human pass over
  `crates/app/ui/index.html`, `crates/app/ui/style.css` and
  `crates/app/ui/src/main.ts`.
