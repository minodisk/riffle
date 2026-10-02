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
