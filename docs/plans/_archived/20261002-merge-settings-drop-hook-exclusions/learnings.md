# Learnings

## Step 1

- SKILL.md's "Entries excluded from the merge automatically" list never named
  the gh / gcloud read-only patterns or a hook, so it needed no change.
- The test's "local with only an excluded entry" case also used
  `Bash(gh issue list *)`, which the plan did not mention; it now uses
  `Write(notes.md)` like the "local adds entries" case, since otherwise the
  entry would be promoted and the no-op assertion would fail.
- On this Windows host, a bare `python` / `node` in Git Bash hung instead of
  failing; run anything needing them through `mise x --`.

## Deferred issues (todo candidates)

- Document that a bare `python` / `node` hangs in Git Bash on the Windows host,
  so scripts go through `mise x --`. No `docs/agents/` guide fits; preferred
  place is a short line under `CLAUDE.md` Development (the hang happens before
  any guide is read), otherwise a new `docs/agents/dev-environment.md`. Check
  first that it is not already in the user's memory. Done when the note exists
  in the chosen place.
