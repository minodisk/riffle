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

# MCP companion: the shown file's closed-eyes judgment and head pose

## Purpose

The meta pane shows the shown file's `Eyes open` row and its `Head pose` row
(yaw / pitch / roll from `crates/core/src/pose.rs`, see
`../_archived/20261007-head-pose/plan.md`), both from the one on-demand
`eyes_of` judgment (`EyesJudgment` in `crates/app/src/commands.rs`, cached
per file in `crates/app/ui/src/eyes.ts`'s `EyesCache`). An MCP client cannot
read either: no tool in `crates/app/src/mcp.rs` carries a closed-eyes field,
so the two `todo.md` items (closed eyes via MCP; head pose next to it) are
open. This work exposes both as one field of `get_view`, so an assistant
following a cull can say "eyes closed, face turned 40° right" from the same
numbers the meta pane shows.

Which case applies: as of 2026-10-08 the index does not store the EAR or the
pose (`origin/main` at `176a96b6`; the "Store the scan's mesh-derived eye
state and head pose in the index" item is still open), so the companion reads
the `EyesCache`. The read goes through one `ViewApi` getter so the source can
be switched to stored values later without touching `companion.ts`'s shape.

## Steps

- [x] Step 1: Add the shown file's eyes judgment and head pose to `get_view`
  - Done when:
    - `get_view`'s `current` carries an `eyes` field for the current photo:
      `{ state: "open" | "closed", probability: <closed, 0..1>, pose: { yaw, pitch, roll } | null }`
      when the file has been judged and a face was found; `null` when it was
      judged and the eyes are unknown (no face, a face under the 60 px floor,
      a failure); the key is absent when the file has not been judged yet
      (the judgment is still in flight, or the folder is view-only, where
      `eyes_of` never runs). `pose` is `null` when the fit failed (including
      a face fit upside down).
    - The companion does not trigger `eyes_of` itself: `get_view` stays a
      synchronous read over live state like every other answer in
      `companion.ts`. `show()` already requests the judgment for every shown
      file, so an absent field means "ask again in a moment".
    - The `get_view` tool description in `crates/app/src/mcp.rs` names the
      field, the sign convention (yaw positive toward the image's right,
      pitch positive up, roll positive clockwise on screen), that
      `probability` is the probability the eyes are closed, and how the
      not-yet-judged (absent) and no-face (`null`) cases appear.
    - `companion.test.ts` covers the cases (object with pose, object
      with `pose: null`, `null`, absent) and that `respond` serializes the
      absent case without the key. `pnpm exec vp test` and `cargo test -p
      riffle-app mcp` pass.
    - `docs/humans/usage.md` "MCP companion" `get_view` row and
      `docs/humans/usage.ja.md` say the field is there and how it reads;
      `README.md` / `README.ja.md`'s MCP section lists no tools, so it needs
      no change unless the implementer finds a line that enumerates what
      `get_view` returns.
    - The two `todo.md` items ("Add the shown file's closed-eyes judgment to
      the MCP companion", "Add the shown file's head pose to the MCP
      companion") are ticked with a one-line note of what landed.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/ui/src/companion.ts`: add to `ViewApi` a getter such as
      `eyes(path: string): Eyes | null | undefined` (the `EyesCache.get`
      contract: `undefined` = not judged, `null` = unknown, `Eyes` =
      judged). In `getView`, map it into `current` as `eyes?: EyesSummary |
      null`, dropping `mesh` (478 points, ~8-10 KB, of no use to a client)
      and keeping `state`, `probability`, `pose` as `EyesJudgment` names
      them. Omit the key (do not set it to `undefined` and rely on JSON) so
      the test can assert `"eyes" in current === false`; `respond`'s
      `JSON`-over-Tauri path drops `undefined` anyway, but an explicit omit
      keeps the TS type honest.
    - `crates/app/ui/src/main.ts`: in the `view: ViewApi` object, `eyes:
      (path) => eyesCache.get(path)`. Nothing else in `main.ts` changes;
      `requestEyes` stays as is.
    - `crates/app/ui/src/companion.test.ts`: extend the `view()` fixture with
      `eyes: () => undefined` and add `getView` cases with a stub returning
      an `Eyes` object (with and without `pose`) and `null`.
    - `crates/app/src/mcp.rs`: only the `get_view` `#[tool(description =
      ...)]` string (and `INSTRUCTIONS`, if the implementer judges the eyes
      cue worth a mention there); no Rust type change, since the frontend
      composes the JSON. Keep `the_tools_are_exactly_the_seven_companion_tools`
      unchanged.
    - Docs: `docs/humans/usage.md` and `usage.ja.md` `get_view` rows in the
      same PR; point the sign wording at the existing `Head pose` paragraph
      rather than restating it.
    - Out of scope: `pose.rs`, `crates/core/src/eyes.rs` constants,
      `candidate.rs`, `eyes_of` / `read_eyes`.

## Trade-offs and risks

- **`get_view` vs `get_photo` vs a new tool.** `get_view` was chosen: the
  judgment exists only for files that have been shown, and `get_view` is the
  frontend-answered tool with a `current` object, so the field is one getter
  and one mapping. Putting it on `get_photo` would need a second bridge
  request (say `get_eyes` with a path) merged into the Rust-built `Photo`
  JSON, and for a path never shown the answer would be absent anyway, which
  misleads more than it helps on a "one photo's details" tool. A new tool
  (`get_eyes`) adds a tool for one field and breaks the seven-tool test for
  no gain.
- **Trigger `eyes_of` or report absent.** Reporting absent was chosen: every
  companion answer is a synchronous read of live state, `show()` already
  requests the judgment for the current file, and the cache allows one
  judgment in flight at a time (`EyesCache.request`), so a companion-driven
  request would either queue behind or fight the UI's own. The downside is
  a client that calls `get_view` right after `show_photo` may see the field
  absent for ~100-150 ms and must call again; the description says so.
- **Field shape.** `probability` is kept as `EyesJudgment` defines it (the
  closed probability), while the meta pane shows `Eyes open: NN%` (`1 - p`).
  The plan keeps the backend name and documents it, so the stored-values
  switch later needs no rename (user approved, 2026-10-08).
- **Parallel session.** If `dlg-store-eyes-pose` lands first and switches
  the meta pane to stored values, the `ViewApi.eyes` getter in `main.ts` is
  the only line to repoint (to whatever map holds the stored judgment);
  `companion.ts` and its tests stay. Check `git log origin/main` for it at
  the start of the step and note the outcome in `learnings.md`.

## Progress

- (2026-10-08) Step 1 complete. `INSTRUCTIONS` in `mcp.rs` was left unchanged.
