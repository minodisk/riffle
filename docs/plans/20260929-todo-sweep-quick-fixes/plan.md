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

# Four small todo fixes: reveal failed mark, live roots, viewer ResizeObserver, lean Exif payload

## Purpose

Four independent, small items in `todo.md` have a chosen fix each. Landing them
in one PR clears them from the todo list and fixes: a `reveal` listing failure
that leaves the folder row unmarked; a volume mounted after launch that never
appears in the tree; a horizontal strip scrollbar that shrinks the viewer
without a redraw; and ten always-`null` Maker note fields sent for every
`IndexedFile` in the folder listing.

## Steps

- [x] Step 1: Land the four fixes and remove their todo.md sections
  - Done when:
    - (1) A `list_subfolders` failure inside `reveal` in
      `crates/app/ui/src/folders.ts` marks the failed folder's row the same way
      `toggle` does (`markFailed(collapse(tree, dir), dir, String(err))`), so
      the row gets the `failed` class and the error as its tooltip, and a later
      successful listing (`setChildren`) clears it.
    - (2) A volume mounted after launch appears at the tree's top level the
      next time the main window gains focus, without duplicating roots already
      shown, and `docs/usage.md` no longer says the roots are read once at
      launch / need a restart.
    - (3) When `#strip`'s horizontal scrollbar appears or disappears and
      changes `#viewer`'s height without a window `resize`, the canvas is
      redrawn (the same work the `resize` handler does).
    - (4) `Exif`'s ten Maker note fields (`focus_mode`, `af_tracking`,
      `af_area`, `drive`, `stabilization`, `exposure_mode`, `metering`,
      `creative_style`, `dro`, `raw_type`) are omitted from the JSON when
      `None`, so `folder_entries` rows no longer carry them as `null`; the
      meta pane and the filter menu behave as before.
    - The four `###` sections are removed from `todo.md`:
      "App: `reveal`'s listing failures in the folder tree don't get the failed
      mark", "App: the folder tree's roots don't pick up a volume mounted after
      launch", "App: a horizontal strip's scrollbar can grow `#film` and shrink
      the viewer without a resize event", "App: the folder listing payload
      carries always-null Maker note fields".
    - `mise run ci` passes.
  - Implementation approach:
    - (1) `crates/app/ui/src/folders.ts`, `reveal`'s `catch`: replace
      `tree = collapse(tree, dir);` with
      `tree = markFailed(collapse(tree, dir), dir, String(err));`. `markFailed`
      is already imported. `render` already draws `node.failed` (class
      `failed`, `row.title`). `reveal` touches `window.__TAURI__` and the DOM
      and has no unit tests, so no new test is expected for this item unless
      one falls out cheaply.
    - (2) `crates/app/ui/src/folders.ts`, `loadRoots`: keep the launch call
      and re-run the `folder_roots` invoke + `addRoots` on the main window's
      focus, using the same scoped listener `main.ts` uses for `resync`
      (`window.__TAURI__.window.getCurrentWindow().listen("tauri://focus", ...)`;
      a global `event.listen` would also receive other windows' focus). Either
      register the listener inside `folders.ts` (matching how it already
      registers `tree-changed`), or export a `refreshRoots()` that `main.ts`'s
      existing focus handler calls; keep the `rootsSettled` promise resolving
      only once. Deduplication is already done by `addRoots` in `tree.ts`, so
      repeated calls add only new volumes. A root that was unmounted is *not*
      removed (out of scope). Do not report a focus-time `folder_roots` failure
      as an error (background refresh, like the `tree-changed` re-list's
      `console.warn`). Do not add a throttle speculatively. Update
      `docs/usage.md` (the "read once at launch, so one mounted afterward does
      not appear until the app restarts" parenthetical) to say a volume
      mounted later appears the next time the window is focused. Also update
      the comment above `toggle` in `folders.ts` ("Roots themselves
      (`loadRoots`) are read once at launch and not refreshed here").
    - (3) `crates/app/ui/src/main.ts`: factor the body of the
      `window.addEventListener("resize", ...)` handler (`draw()` + the
      `scheduleCropForResize()` when `zoomed`) into a small function and call
      it both from the `resize` listener and from a
      `new ResizeObserver(...).observe(viewer)` on `#viewer`. The observer
      fires once on `observe()`; `draw()` already clears and returns when
      `shown === null`, so that is harmless. Keep both listeners unless the
      double draw on a window resize measurably matters. No unit test
      (`main.ts` is not unit-tested); `style.css` is not expected to change.
    - (4) `crates/app/src/exif.rs`: add
      `#[serde(skip_serializing_if = "Option::is_none")]` to the ten Maker note
      fields of `pub struct Exif`. Add a unit test in `exif.rs`'s `mod tests`
      that `serde_json::to_value(exif(&Shot::default()))` has no `focus_mode`
      key (etc.) and still has `camera: null`. The frontend `Exif` interface in
      `crates/app/ui/src/exif.ts` never lists these fields (the meta pane reads
      them off `Metadata`), so no TypeScript change is needed; confirm with a
      grep that nothing in `ui/src` reads those ten names off an `Exif`, and
      that `mcp.rs` / `companion.ts` do not depend on them.
    - `todo.md`: delete the four `###` sections (each with its `#### TODO`
      list) listed above; leave every other section untouched.

## Trade-offs and risks

- Roots refresh trigger: focus (chosen) vs. each `reveal`. Focus covers a card
  inserted while the app is in the background; `reveal` already adds an
  unknown root on its own (`rootOf(path)` + `addRoots`).
- The focus refresh only appends: an unmounted volume stays listed until
  restart. Removing roots is out of scope.
- `ResizeObserver` + `resize` both firing on a window resize means two
  `draw()`s per frame while dragging the window edge; kept unless measured to
  matter.
- Item (4) changes the wire shape of `Exif` in `folder_entries`; the frontend
  type never had the fields, so the only risk is an undiscovered consumer,
  which the grep rules out.

## Progress

- (none yet)
