# Learnings

## Step 1

- The `ai-coauthored` label did exist on GitHub (description "Co-authored with AI", colour `#ededed`); it was deleted with `gh label delete ai-coauthored --yes`, which also strips it from past PRs (accepted by the user). `gh label list` no longer shows it.
- `create-pr.sh` had only two references (usage text and the `gh pr create` flag); both removed.

## Step 2

- lychee 0.24.2 (`mise use lychee@0.24.2`) with `--offline --include-fragments`
  still checks relative file links and `#heading` anchors; a broken anchor
  appended to `README.md` made it fail. No need to drop `--offline`.
- File set: all root `*.md`, `docs/**/*.md` and `.claude/**/*.md` pass, except
  `docs/plans/review-history/`, whose review logs quote a README anchor
  (`#running-the-app`) as it existed at review time. That directory is excluded
  with `--exclude-path` since those records are historical and should not be
  rewritten.
