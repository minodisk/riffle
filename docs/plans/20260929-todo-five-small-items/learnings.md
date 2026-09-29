# Learnings: todo-five-small-items

## Step 1

- The rule went in as a bullet of the "Do not hand-merge generated artifacts."
  list in `.claude/agents/pr-conflict-resolver.md`, with two fenced `bash`
  checks: `grep '^### ' todo.md | sort | uniq -d` (duplicated headings) and
  `git diff origin/main -- todo.md | grep '^[-+]### '` (the headings the
  branch adds or removes relative to main; each must be intentional). The
  second is the concrete form of "compare the branch's deleted headings
  against `origin/main`" and also works mid-rebase.
- The Edit tool wrote under `.claude/` without a sandbox override; only Bash
  commands writing there would need `dangerouslyDisableSandbox`.
- Removing a `todo.md` section with `sed -i 'N,Md'` left a doubled blank line
  the first time; check the seam after deleting.
- The user-approved `.claude/settings.json` change (`Bash(gh pr view:*)`) rode
  along in this step's commit as instructed.

## Step 2

- Comment-only change: the header's first paragraph now names the folder as
  the one right-clicked in the folder tree, and the second says the output
  goes to a sibling of "the right-clicked folder" instead of "the picked
  folder". No code touched.
- The doubled-blank-line seam from Step 1 happened again when removing the
  `todo.md` section with `sed -i 'N,Md'` (the range stopped at the section's
  last text line, leaving its trailing blank next to the previous section's);
  check the seam every time.

## Step 3

- `anchorAfterFilter` got an optional `previous` list. When it is given and
  the anchor is missing from the new `allFiles` (a `Set` lookup), the picker
  walks `previous` forward, then backward, for the first path that is both
  still listed and passing, ignoring whether the anchor itself passes (the
  kept `ratings` / `flags` / `labels` usually make it pass). Only `resync`
  passes it: it captures `files` (the on-screen filtered list) before the
  `list_arw` call and hands it through a new fourth `refilter` parameter.
  `files` is only ever reassigned, never mutated, so the captured reference
  stays the pre-listing list.
- The plan wrote `refilter(anchor, force, keepScroll)`, but the real order is
  `refilter(anchor, keepScroll, force)`; `resync` now calls
  `refilter(target, true, false, previous)`, keeping its old
  `keepScroll = true, force = false`.
- Verified in code: `mustReshow(force, files[index], anchor)` is
  `current !== anchor`, so landing on the neighbour triggers `show()`.
- `pnpm` is not on the Bash tool's PATH; the Vitest run went through
  `mise run ci`.
- The `todo.md` section removal (`sed -i '1190,1197d'`, heading through the
  blank line after its TODO) left a clean seam this time.

## Deferred issues (todo candidates)

- **Pending manual check (Step 3, deleted focused file keeps a neighbour).**
  Not run: the GUI cannot be driven from the implementing session. On
  Windows (and macOS if available): open a folder of RAW files, focus a file
  in the middle of the strip, delete that file in Explorer / Finder, and
  switch back to the app. Expected: the strip stays on the deleted file's
  next file (or the previous one if it was the last), not the first file,
  and the preview shows that file. Repeat with a filter active whose next
  neighbour is filtered out; expected: focus lands on the next passing file.
  The Step 3 checkbox was ticked on the automated criteria (Vitest cases in
  `crates/app/ui/src/filter.test.ts`, `mise run ci`). Files:
  `crates/app/ui/src/main.ts` (`resync`, `refilter`),
  `crates/app/ui/src/filter.ts` (`anchorAfterFilter`).
