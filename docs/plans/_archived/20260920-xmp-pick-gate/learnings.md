# Learnings

## Step 1: Gate the pick on `sidecarFormat` in the frontend

- The gate lives in `applyRating` (`crates/app/ui/src/main.ts`), not in
  `strip.ts`: every path that fills `picks` goes through `applyRating`, and
  `passes`, `judge`, `undo` and `refilter` all read `picks.has(path)`, so one
  `const kept = effectivePick(pick, sidecarFormat)` fixes the flag dot, the
  "picked" filter and undo together. Gating the render alone would have left
  `picks` true. `strip.ts` stayed untouched.
- The rule is `crates/app/ui/src/pick.ts` `effectivePick(pick, format)`, a pure
  module so vitest (`environment: "node"`) can import it; `main.ts` and
  `strip.ts` are not importable there. The `case "pick"` early return now calls
  `effectivePick(true, sidecarFormat)` instead of its own `!== "dop"` literal.
- The backend rule in `crates/app/src/commands.rs` `set_rating` already read
  `pick && rating != Some(-1) && format == SidecarFormat::Dop`, format check
  included, so no Rust change was needed. The helper deliberately folds in only
  the format check; the reject exclusion stays in the `judge` callbacks and the
  backend.
- Launch ordering confirmed by reading `main.ts` rather than by logging: the
  `sidecar_format` invoke fires at module load (~line 1423), while the first
  `folder_entries` is two round trips behind it (`sort_order` at ~1646
  `.finally(reopenLastFolder)` -> `last_folder` -> `folder_entries`). So the
  format is known before the first `applyRating`, and no speculative
  `refreshEntries()` was added.

## Step 2: Close the todo and record the residual

- Removed the "App: a pick kept across a sidecar format switch still shows its
  flag dot in the wrong format" heading and its TODO list from `todo.md`.
- A guide entry was judged worthwhile and added to `docs/agents/tauri-app.md`
  ("A pick is only meaningful while `.dop` is selected"), right after the
  `reset_sidecars` entry: the two are the backend and frontend halves of the
  same rule, so the reader of `reset_sidecars`'s doc comment
  (`crates/app/src/index.rs`) finds the frontend counterpart next to it.

## Deferred issues (todo candidates)

- (none)
