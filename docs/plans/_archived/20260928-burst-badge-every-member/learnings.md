# Learnings

## Step 1

- `paintBurst` in `crates/app/ui/src/strip.ts` was the only place deciding the
  badge text, and `highlight()` repainted it only so the badge followed
  `current`. With the text now a pure `burstBadge` in `burst.ts` that ignores
  `current`, that call in `highlight()` was dropped; `createCell` and
  `setBurst` still paint the badge.
- No CSS change: `.cell span.count` is right-anchored (`right: 2px`,
  `width: auto`, `bottom: 26px`), so a longer badge like `12/15` grows
  leftward across the 144 px image box, away from the sharpness bar
  (`left: 2px`) and the candidate icon (bottom-left). It was not checked in
  `mise run tauri:dev` (no GUI here); the pending by-hand item in `todo.md`
  now asks for it.
