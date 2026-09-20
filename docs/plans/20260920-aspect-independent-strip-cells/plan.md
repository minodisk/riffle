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

# Make the filmstrip cell image box aspect-independent

## Purpose

`.cell img` in `crates/app/ui/style.css` is a fixed 144x96 box, matching the
current 404x270 (3:2) thumbnail pipeline. A body whose IFD0 preview is a
different shape (4:3, say) would still fit thanks to `object-fit: contain`,
but its width would be capped by the 96px height (128x96), wasting cell
space. Turning the box into the 144px square its own comment already calls
the "footprint" makes the layout aspect-independent: 3:2 thumbnails render
pixel-for-pixel as today, and any other aspect fills the cell width or
height instead. The virtual list (`--cell-height`, `strip.ts`) is untouched.

## Steps

- [x] Step 1: Make `.cell img` a 144x144 box and keep the placeholder grey off the loaded thumbnail
  - Done when:
    - `.cell img` in `crates/app/ui/style.css` is `width: 144px; height: 144px` with `object-fit: contain`, still centred at (72px, 72px) via `translate(-50%, -50%)`; `.cw`, `.ccw` and `.half` are unchanged
    - A loaded 3:2 thumbnail (flat, `cw`, `ccw`, `half`) looks identical to before the change: 144x96 (or 96x144) with no grey band above/below or beside it. Verified by eye in the running app on an existing ARW/DNG folder
    - A loaded non-3:2 thumbnail fills the full 144px width (landscape) or 144px height (portrait). If no such file is at hand, verified by temporarily pointing an `<img>` with the same CSS at a 4:3 image in the dev server, or by reasoning from `object-fit: contain` in the PR description
    - The not-yet-loaded placeholder is still visibly grey (`#2a2a2a`) on `#1c1c1c`; no grey shows outside a loaded thumbnail
    - The comment above `.cell img` ("The image box is 144x96 ...") describes the new rule: the box is the 144px square footprint, `object-fit: contain` scales any aspect into it, a 3:2 thumbnail lands at 144x96 exactly as before, and the quarter turns stay within the square so the cell height the virtual list depends on does not change
    - The "App: filmstrip cell geometry assumes 3:2 thumbnails" section (heading through its TODO list) is removed from `todo.md`
    - `--cell-height`, `--cell-gap`, `.cell` and `crates/app/ui/src/strip.ts` are not modified
    - `mise run ci` passes
  - Implementation approach:
    - `background: #2a2a2a` on `.cell img` is the placeholder for cells whose thumbnail has not arrived yet (`createCell` in `strip.ts` appends an `<img>` without `src`; `src` is set only on payload arrival, and cells are recreated rather than reused, so `img:not([src])` is exactly "placeholder or failed"). With a square box that background would show as 24px bands around a loaded 3:2 thumbnail, so it must not stay on the loaded image. Scope it to the placeholder with a `:not([src])` rule carrying **only** the background, so the placeholder is the full 144px square and no 3:2 constant survives (the shape decided by the user; see Trade-offs)
    - Keep the change to the `.cell img` block, one new rule immediately after it, and the comment. Match the surrounding comment style (prose sentences, wrapped at the same width)
    - Do not add a background to `.cell`: `.cell.current` and `.cell.failed` set their own, and a base `.cell` background would tint the file-name strip and the badge area too
    - Everything committed is in English

## Trade-offs and risks

- **Placeholder shape once the background moves to `img:not([src])`.** The
  user chose the square placeholder: the `:not([src])` rule carries only
  `background`, so the unloaded/failed placeholder grows from 144x96 to the
  full 144x144 square. This keeps the rule to one line and leaves no
  residual 3:2 constant, at the cost of a minor visible change while a
  folder is scanning, and of covering more of the `#402020` tint on
  `.cell.failed`. The rejected alternative was to pin `height: 96px` in the
  `:not([src])` rule to keep the placeholder pixel-identical to today.
- **`img:not([src])` relies on `src` never being cleared.** True today
  (`strip.ts` recreates cells and only ever sets `src`). If a future change
  starts reusing cells and clearing `src`, the placeholder simply
  reappears, which is the desired behaviour anyway.
- **No automated check of the rendering.** There are no CSS/visual tests;
  the acceptance is by inspection in the running app. Record what was
  looked at in the PR description.
- **Rejected: moving the background to `.cell`.** It would tint the
  file-name strip and badges and interact with `.cell.current` /
  `.cell.failed`, which is a larger visual change than the task asks for.

## Progress

- (none yet)
