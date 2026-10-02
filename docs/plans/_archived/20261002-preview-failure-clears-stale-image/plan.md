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

# Clear the main preview when the selected file's preview fails

## Purpose

Selecting a file whose embedded preview cannot be shown (for example
`D:\Photos\samples\NEF\NIKON_D70_Nikon.nef`, whose strip cell reads "panic
while encoding the thumbnail" and whose JPEG makes `createImageBitmap` throw
`InvalidStateError: The source image could not be decoded.`) leaves the
previous file's image in the main preview. The title bar, the meta pane and the
status note all switch to the new file, so the stale image reads as the new
file's picture.

Cause (`crates/app/ui/src/main.ts`): `shown` holds the last decoded bitmap and
is deliberately kept across `show()` so the canvas is not blanked while the
next preview decodes (`draw()` draws `shown` whatever its `seq`; only
`drawZoom` and `drawFaceMarks` guard on `shown.seq === seq`). The two failure
branches for the current `seq` only set the status note and return:

- the worker's decode error in the `worker.addEventListener("message", ...)`
  handler (`bitmap === undefined` branch);
- the `preview` command rejecting (backend `Err`, or the
  `unknown preview payload kind` throw) in `requestPreview()`'s `.catch`.

After this, a file whose preview fails shows an empty preview area with a
note that it could not be shown, and the next normal file draws as before.

## Decisions

- Failure-state shape: option A (user choice, 2026-10-02). Extend `empty.ts`
  with a `"preview-failed"` `EmptyState` and its text constant, and show it in
  the existing `#empty` overlay over the cleared canvas.
- The failure flag is per-`seq` and resets at `show()`, so the overlay
  vanishes as soon as the user moves to another file.

## Steps

- [x] Step 1: Clear `shown` on a preview failure of the current file and show the failure state
  - Done when:
    - In `crates/app/ui/src/main.ts`, both failure branches for the current
      `seq` (the worker `message` handler's `bitmap === undefined` branch and
      `requestPreview()`'s `.catch` when `current === seq`) close and null
      `shown`, then `draw()`, so the canvas is cleared. The stale-`seq`
      branches keep their current behavior (drop silently / re-request as
      today, per the `docs/agents/tauri-app.md` "one token" note: the
      post-await stale branch of the worker path must not re-issue the
      request).
    - The error text still reaches the meta pane's status block the way it
      does today (`setStatus(...)`), so the meta pane behavior is unchanged.
    - The preview area shows the `"preview-failed"` state in the existing
      `#empty` / `.empty` overlay (centered `--muted-foreground` text, like the
      no-files / filtered hints), with a short sentence in the style of
      `NO_FILES_TEXT` / `FILTERED_TEXT` (e.g. "The preview of this file could
      not be shown."). The raw error message is not duplicated in the overlay.
    - Moving from the failed file to a normal file shows it normally: the
      failure flag is reset in `show()` and the overlay returns to `"none"`.
    - Fast navigation: a failure for a stale `seq` changes nothing on screen;
      a failure for the current `seq` clears whatever `shown` held, even if
      it belonged to an earlier file (that is the bug). Confirm by reading
      the code paths that no path can draw an old `shown` after the clear
      (`draw()`, `drawZoom()`'s placeholder, `drawFaceMarks`).
    - `crates/app/ui/src/empty.test.ts` covers the new `emptyState(...)`
      branch and keeps the existing three states winning when they apply.
    - `mise run ci` passes (`vp check` / `vp test` / lint / fmt).
    - Pending manual check (real app, Windows, not a blocker for merge):
      open `D:\Photos\samples\NEF\`, view a normal NEF, then
      `NIKON_D70_Nikon.nef`: the main preview no longer shows the previous
      NEF; the failure text appears; paging on to a normal file shows it;
      paging back and forth quickly across the bad file never leaves an older
      file's image on screen; `z` (1:1) and Compare on the bad file do not
      throw.
  - Implementation approach:
    - Keep the change inside `main.ts`, `empty.ts` and `empty.test.ts`
      (`style.css` only if a rule is actually needed; the `#empty` click
      handler only acts on `"no-folder"`). Do not touch `worker.ts` (its error
      path already posts `{ seq, error }`) or the Rust `preview` command.
    - The clear goes where the error is handled, not into `draw()`: `draw()`
      must keep drawing an older `shown` between `show()` and the next
      successful decode, or every page turn would flash black. Do not add a
      `shown.seq === seq` guard to `draw()`.
    - Mirror the existing clear sites exactly
      (`shown?.bitmap.close(); shown = null; draw();` as in `refilter`'s
      empty branch and `openDirectory`).
    - The `preview` command's `Err` and the decode error share the fix, since
      both land in the same two spots. `requestMetadata()`'s catch comment
      ("the preview request reports the error itself") stays true.
    - The compare path (`loadCompare`) and the 1:1 crop path (`requestCrop`)
      already clear / do not reuse the old frames on failure; leave them
      alone.
    - Overlap check: the in-flight plan
      `docs/plans/20261002-error-row-exif-tree-refresh/plan.md` touches the
      folder-menu `switch` in `main.ts`, `context.ts` and `folders.ts`; none of
      `requestPreview`, the worker handler, `draw()` or `empty.ts`, so at most
      a trivial `main.ts` merge.

## Trade-offs and risks

- A failure arriving for a stale `seq` is still dropped without touching the
  screen; the fix only acts for the current `seq`. This is correct for fast
  navigation (the newest request will land its own result or failure).
- After a failure, `shown` is null, so moving to the next file shows a blank
  canvas for the few ms its decode takes (no placeholder image to keep).
- The strip's `failed` cell styling (`#402020`, `.reason`) is per-cell and is
  not reused on the canvas: a canvas-drawn error box would need token reads
  via `THEME` and a new layout; the `#empty` overlay already exists for
  "nothing to draw here" text.

## Progress

- (2026-10-02) Step 1 complete
