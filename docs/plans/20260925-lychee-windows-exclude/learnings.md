# Learnings

## Step 1

- lychee 0.24.2 on Windows reports input paths with backslashes, so the
  `--exclude-path` regex must not assume `/`. `docs.plans.review-history`
  works under the Git Bash `bash -c` that mise uses for tasks: with it the
  run reports 497 total links, 51 OK, 0 errors.
- Sanity check: the same flags against a scratchpad file holding
  `[x](#nope)` still report 1 error, so fragment checking is intact.
- `mise x -C <dir> -- lychee <relative file>` resolves the relative input
  against `<dir>`, not the shell's cwd; pass an absolute path instead.
