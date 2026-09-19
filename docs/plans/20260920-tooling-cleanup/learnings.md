# Learnings

## Step 1

- The `ai-coauthored` label did exist on GitHub (description "Co-authored with AI", colour `#ededed`); it was deleted with `gh label delete ai-coauthored --yes`, which also strips it from past PRs (accepted by the user). `gh label list` no longer shows it.
- `create-pr.sh` had only two references (usage text and the `gh pr create` flag); both removed.
