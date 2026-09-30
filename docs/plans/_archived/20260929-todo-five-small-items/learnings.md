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

## Step 4

- `relation`, `rebase` and `renameFolder` in `crates/app/ui/src/tree.ts` now
  take a trailing `ignoreCase = false` and compare through `fold`; `rebase`
  still joins the caller's own remaining segments onto `newDir`, so the
  spelling below the renamed folder is preserved. `respell` passes its
  `ignoreCase` to `rebase`, and `folders.ts`'s `renamed` passes the module
  flag to `renameFolder` and to each of its four `rebase` calls (`current`,
  `cursor`, the selection and its anchor).
- Untouched callers, case-sensitive by decision (option (a)):
  `crates/app/ui/src/main.ts`'s post-rename `rebase(openDir, path, newPath)`
  in `renameFolder`, and `crates/app/ui/src/trash.ts`'s `relation` calls in
  `restoredInto` and `opensTarget`.
- With `ignoreCase`, a case-only rename (`/home/me/a` to `/home/me/A`) is not
  treated as stale: `rebase(path, oldPath, newPath)` matches first, so the
  node moves with its state. A pre-existing node at the new path in another
  case (`D` when renaming `a` to `d`) is dropped as stale; both are tested.
- Hit the Step 5 pitfall while writing the tests: a Bash-tool heredoc fed to
  `python -` turned the test file's `\\` into `\`, so the replacement's
  anchor text no longer matched (the assertion caught it before any write).
  Writing the script with the Write tool and running it from the scratchpad
  worked.

## Step 5

- Re-confirmed on Windows 11: the string `D:\\Photos \\?\C:\x "a\\b" \\\\server`
  written to the scratchpad through a `cat > f <<'EOF'` Bash-tool heredoc
  landed as `D:\Photos \?\C:\x "a\b" \\server` (every `\\` halved, single
  backslashes intact); the same string through the Write tool landed byte for
  byte, and `diff` showed the one line differing.
- It is not heredoc-specific: `printf '%s\n' 'a\\b'` and `echo 'a\\b'` in the
  Bash tool also wrote `a\b`, so the halving happens to the command string
  itself. A `printf '%s'` workaround (the plan's fallback idea) does not help;
  the rule says Write / Edit or a script file written by them. Step 4's
  `python -` heredoc hit above is the same pitfall in practice.
- Added the rule as "On Windows, the Bash tool halves doubled backslashes,
  even in a quoted heredoc (Hit)" right after the Python-heredoc item in
  `docs/agents/tauri-app.md`. Since the quoted-heredoc hand-over in
  `.claude/skills/delegate/SKILL.md` is affected too, it got a two-line
  cross-reference there.

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
- **Step 4 scope: `main.ts` / `trash.ts` path comparisons stay
  case-sensitive.** Source: this plan's Step 4 (option (a) in "Trade-offs and
  risks"). `crates/app/ui/src/main.ts`'s `renameFolder` reopens the folder
  through `rebase(openDir, path, newPath)` and `crates/app/ui/src/trash.ts`'s
  `restoredInto` / `opensTarget` call `relation`, all without `ignoreCase`, so
  on macOS / Windows an `openDir` that differs in case from the tree key would
  not be rebased (the open folder not reopened after a rename) or matched.
  Rare, since both spellings come from the same tree / `list_arw` listings;
  threading `folders.ts`'s platform flag (or an exported equivalent) through
  would close it.
