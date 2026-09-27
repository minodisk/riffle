# Learnings

## Step 1

- Docs-only change. The new `### Lightroom` subsection sits before
  `### Lightroom Classic` in both READMEs, matching the checklist order. It
  names Lightroom 9.5.1 on Windows and states no color-label matching rule,
  since whether non-Classic Lightroom matches by `photoshop:LabelColor` or by
  name was not isolated during the manual check.
- Scope extension (approved by the user mid-step): Lightroom Classic is
  ticked too, after the user verified a Riffle-written reject
  (`xmpDM:good="False"`, no `xmpDM:pick`), pick and unflagged file display
  correctly. The todo item about the Lightroom Classic pick/reject was removed.
  Riffle does not need to write `xmpDM:pick`.
- In this worktree `pnpm` is not on the bash PATH, so `mise run fmt` failed
  with "vp not found" until `mise exec -- pnpm install --frozen-lockfile` ran.
- Second scope extension: the `verified` strings in `crates/core/i18n/en.json`
  and `ja.json` (and the example in `crates/core/i18n/README.md`) now name
  Lightroom Classic 15.5.1 on Windows, after the user checked both UI
  languages' default label sets. `verified` is display-only metadata, so no
  Rust test pins its text.
