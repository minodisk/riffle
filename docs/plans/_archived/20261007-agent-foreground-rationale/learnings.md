# Learnings

## Step 1

- `merger.md` holds two counters, but they tick on different exit codes
  (`pr_wait_timeouts` on exit 2, `pr_status_failures` on exit 1), so the new
  rationale names each counter with its own exit code instead of copying
  `pr-runner.md` §4's "tick on exit 2" wording verbatim.
- In `merger.md` the incident sentence names `pr-runner` explicitly ("the one
  time this was tried `pr-runner` handed ..."), since "the runner" would be
  ambiguous inside the merger's own file.
- A stray `cat > file` with no input in a shell one-liner blocks on stdin until
  the Bash tool times out; keep heredocs attached only to the command that
  reads them.
- Inside the sandbox, `mise run ci`'s vitest run died on a Windows
  `stat` UNKNOWN error (errno -4094) under `node_modules/.pnpm/.../@vitest/snapshot`,
  after lint had passed. This was environmental: the same `mise run ci` passed with
  the sandbox disabled (39 test files, 570 tests).

## Deferred issues (todo candidates)

- `merger.md`'s post-merge paragraph still says backgrounding is "impossible
  for the same reason as step 1"; the corrected reason makes it ruled out, not
  impossible. Left as is to keep this change surgical (see this plan's
  "Trade-offs and risks"). File: `.claude/agents/merger.md`. Done when the
  post-merge paragraph no longer calls backgrounding "impossible" and its
  wording agrees with step 1 and with `pr-runner.md` §4.
