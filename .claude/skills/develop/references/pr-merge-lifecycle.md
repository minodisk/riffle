# PR merge lifecycle

The same procedure applies to every PR in `develop` and to a standalone `/pr`.
When `pr-runner` returns `ready` or `merged`, start `merger` without skipping
the post-merge work, even when the PR is already merged. Do not specify `model`;
leave it to `merger`'s frontmatter.

```
Agent(
  subagent_type: "merger",
  run_in_background: false,
  prompt: "PR number: {number}")
```

**Do not set `run_in_background: true`.** Waiting for CI and waiting for
post-merge are carved into the foreground inside the agent.

`merger` handles the approval judgement for changed paths (with
`.claude/skills/merge/scripts/check-merge-approval.sh` as the source of truth),
the squash merge, waiting for the post-merge workflow, syncing main, and
cleaning up the branch. It does not bypass protections with `--admin`, and
branch deletion is left to `merger`'s existing cleanup. All that comes back is a
count summary and the URLs of failed runs — no run tables, no wait logs. Main
does not take on root-cause investigation: do not go fetch details that were not
returned, via `gh run list` or otherwise, outside of "The minimum corroboration
for R6" below.

Branch on the terminal state of the return value:

| Terminal state                             | What to do |
| ------------------------------------------ | ---------- |
| `MERGED`                                   | Report the merge commit SHA, whether this was a fresh merge or a recovery of an already-merged PR, and the post-merge summary, in one line. `develop` moves on to the next Phase / Step; a standalone `/pr` finishes normally |
| `NEEDS_APPROVAL`                           | Go to "Handling `NEEDS_APPROVAL`" below |
| `NOT_READY`                                | Go to "Handling `NOT_READY`" below |
| `POST_MERGE_FAILED` / `POST_MERGE_TIMEOUT` | **Do not proceed.** Do the minimum corroboration below, then report the failed run URL and how long it waited. Add that the PR is merged but main sync and branch cleanup have not run |
| `FAILED`                                   | **Do not proceed.** Do the minimum corroboration below, then report the reason, the step it stopped at, and whether it merged |

Ask the user for direction on `POST_MERGE_*` / `FAILED`. Only when post-merge
succeeded (or had no runs to wait on) and main sync and branch cleanup also
succeeded does it finish normally as `MERGED`.

## The minimum corroboration for R6

`merger` is contractually not an investigator, so before asserting a
`POST_MERGE_*` / `FAILED` reason to the user, main runs one or two read-only
commands to corroborate the summary. No root-cause investigation, no reruns, no
re-merging.

1. For every terminal state, check once whether the PR merged and what its
   checks look like now.

   ```bash
   gh pr view <PR number> --json state,mergeCommit,mergeStateStatus,statusCheckRollup
   ```

2. If `POST_MERGE_FAILED` came with a failed run URL, check that URL once. Only
   when `POST_MERGE_TIMEOUT` came with no URL, check the run status once by
   merge commit SHA. This check covers the post-merge state only; do not use it
   to infer whether a deploy fired.

   ```bash
   gh run view <failed run URL> --json status,conclusion,url
   gh run list --commit <merge commit SHA> --limit 20 --json status,conclusion,url
   ```

Only when `merger`'s summary and the corroboration agree do you report the
reason, the merge state, and the outstanding post-merge work as established
fact. When they disagree, or when the corroboration alone does not support the
reason, do not re-dispatch `merger`: report only the established facts and the
discrepancy with the summary, and stop. Do not assert a cause; ask the user for
direction.

## Handling `NEEDS_APPROVAL`

`merger` cannot ask the user questions. When there are changes that need
approval, or unclassified paths, it returns to main **without merging**.

1. Report the approval reason, the unclassified paths, and a summary of the
   tallies and the changes, in prose, and stop. Do not raise `AskUserQuestion`
2. Only on resuming after explicit user approval, re-dispatch with an
   approved flag

```
Agent(
  subagent_type: "merger",
  run_in_background: false,
  prompt: "PR number: {number}
Approved: yes (the user reviewed {summary of the changes} and approved the merge)

Skip the approval judgement and go from the merge through post-merge waiting, main sync, and branch cleanup.")
```

After re-dispatch, follow the same return-value branching. If the user merges it
themselves, start `merger` once they say they are done and run post-merge onward
to completion as an already-merged recovery. If they say to abort, stop. A
malfunction in the approval judgement itself is `FAILED` and stops fail-closed.

## Handling `NOT_READY`

To absorb changes between `pr-runner` returning `ready` and `merger` looking at
the state, retry automatically exactly once.

1. Re-dispatch `pr-runner` in existing-PR mode
2. On `ready` / `merged`, start `merger` again. `closed` / `aborted` /
   `needs_discussion` follow the caller's usual branching
3. If the restarted `merger` returns `NOT_READY` again, report the observed
   state and stop

Do not try a third time.
