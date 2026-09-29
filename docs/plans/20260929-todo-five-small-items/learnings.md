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
