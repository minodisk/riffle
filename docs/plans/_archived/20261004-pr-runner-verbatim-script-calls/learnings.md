# Learnings

## Step 1

- The `.claude/settings.json` allow rules for skill scripts are prefix
  wildcards (`Bash(bash .claude/skills/pr/scripts/*)` and siblings), so the
  paragraph says the bare `bash .claude/skills/...` command lines are what
  they allow, rather than claiming an exact-line match the way `merger.md`
  step 6 can for its two fixed commands.
- The old "Do not compose a compound command" sentence was folded into the
  new list instead of kept as a separate paragraph, so §4 has one rule for
  the shape of script calls.
