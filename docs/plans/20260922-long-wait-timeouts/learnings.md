# Learnings

## Step 1

- Motivation: in this harness the Bash tool's default foreground timeout was
  observed to be 120 s, not the 6 minutes the agent and script docs assumed.
  `wait-pr-actionable.sh` (about 5 minutes real time) was moved to the
  background, and `merger` / `pr-runner` then improvised `sleep` / `until`
  polling and left background shells lingering after the hand-back.
- The fix is verified only statically here (docs and script comments). A real
  `/pr` or `/merge` run must be observed to confirm that the `merger` /
  `pr-runner` subagents end with no lingering background tasks, and that
  `TaskStop` from a subagent reaches auto-backgrounded shells.
- The skill docs (`develop`, `pr`, `merge`, `release`, `pr-merge-lifecycle.md`,
  `develop/README.md`) delegate the long waits to `merger` / `pr-runner` and
  contain no direct long-running foreground call, so they are unchanged.
