# Learnings

## Step 1

- The `viewOnly` guard was hoisted into `judge()` (kept in `record()` too, for
  the MCP companion's `view.judge`), so `judge()` returns `paths.length`
  whenever it addresses files and `0` for no files or a view-only folder.
- The advance decision moved into the pure `shouldAdvance` in
  `crates/app/ui/src/advance.ts`; `runAction` only gathers its inputs.

## Deferred issues (todo candidates)

- Pending manual check (not verified; the user's): on Windows / macOS with
  the desktop app and `Auto-advance after a star, reject or pick` on, check
  `3` twice on a file (the second press moves on, no undo entry, status line
  untouched), `x` on a rejected file and `p` on a picked file (both move on),
  the last file (stays), a 2-file selection (no advance), Compare (no
  advance) and a JPEG-only folder (nothing happens). Step 1's checkbox was
  ticked on the automated criteria (`advance.test.ts`, `mise run ci`).
  Basis: plan.md Step 1 hand-check; files `crates/app/ui/src/main.ts`,
  `crates/app/ui/src/advance.ts`.
