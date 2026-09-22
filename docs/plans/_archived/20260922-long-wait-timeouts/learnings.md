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

## Deferred issues (todo candidates)

- Confirm the timeout fix on a real `/pr` or `/merge` run. The fix is verified
  only statically. Watch `merger` and `pr-runner` through their waits and
  confirm they pass `timeout: 600000` (no "moved to the background" message),
  do not improvise `sleep` / `until` polling if a command is backgrounded
  anyway, and `TaskStop` any background task of their own before handing
  back, so `ListAgents` shows them completed with nothing running. Done when
  at least one such run is observed clean; if not, adjust
  `.claude/agents/{merger,pr-runner}.md`.
