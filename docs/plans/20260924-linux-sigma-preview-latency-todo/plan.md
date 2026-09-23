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

# Record the Linux preview latency for SIGMA fp L

## Purpose

After #383 (resize oversized previews in the decode worker on Linux) and #385
(6 MP Linux preview pixel limit), the main view for SIGMA fp L DNGs displays
on Linux (WebKitGTK), but visibly later than for other files. Nothing in the
code is wrong; the cost is structural (no mid-size embedded JPEG, so the
full-size ~60 MP JPEG is shipped over IPC and resized on decode). This work
records the observation, its cause, the measurement already taken, and the
candidate fixes in `todo.md`, so the follow-up is not lost. Docs only; no code
changes.

## Steps

- [x] Step 1: Add a `todo.md` item for the Linux-only SIGMA fp L preview latency
  - Done when:
    - `todo.md` has a new `### App: ...` item (placed next to the related
      "App: SIGMA fp L strip may decode the full-size JPEG per thumbnail"
      item) that states:
      - the observation: on Linux (WebKitGTK) the main view for SIGMA fp L
        DNGs appears noticeably later than for other files, seen after #383
        and #385;
      - the cause: SIGMA fp L has no mid-size embedded JPEG, so the `preview`
        payload is the 9520x6328 (~60 MP, ~28 MB) full-size JPEG; on Linux
        the decode worker (`crates/app/ui/src/worker.ts`) must pass
        `createImageBitmap` resize options to fit under
        `PREVIEW_PIXEL_LIMIT` (6 MP, `crates/app/src/commands.rs`), because
        WebKitGTK draws transferred bitmaps >= ~6.87 MP transparent;
      - the measurement: a MiniBrowser run showed ~200-400 ms per resized
        decode of a synthetic 60 MP JPEG, on top of the IPC of the ~28 MB
        payload; the real app is unmeasured (label the figure as synthetic);
      - a `#### TODO` checklist with the candidate fixes, none decided:
        measure first on the real app with `Timing logs` on (the
        `page invoke=… decode=… total=… keypressToPixels=…` line) to split
        invoke vs decode; prefetch/decode neighbouring pages ahead; have the
        backend downscale and cache a mid-size JPEG for files lacking one;
      - cross-references (not duplicates) to "App: SIGMA fp L strip may
        decode the full-size JPEG per thumbnail" (same root cause, strip
        side; the backend-downscale fix would serve both) and to
        "App: unmeasured end-to-end per-page latency" (the general
        measurement; this item is the Linux/SIGMA-specific case of it).
    - `mise run ci` passes.
    - The commit follows Conventional Commits, e.g.
      `docs(todo): record the Linux preview latency for SIGMA fp L`.
  - Implementation approach:
    - Match the existing item shape in `todo.md`: `### App: <title>`,
      a prose paragraph with the facts and `Files:` references, then
      `#### TODO` with `- [ ]` entries. Wrap prose at the same width as the
      neighbouring items.
    - Files changed: `todo.md` only. Do not touch
      `crates/app/src/index.rs` or any code.
    - The existing "SIGMA fp L strip" item's TODO already says "consider
      downscaling the full-size JPEG for the preview tier"; in the new item
      say that fix is shared rather than restating it as a separate plan.

## Trade-offs and risks

- New item vs extending an existing one: the user chose a separate item that
  cross-references both related items.
- The 200-400 ms figure is from a synthetic JPEG in MiniBrowser, not from
  the app; the item must label it as such.

## Progress

- (none yet)
