---
name: merge
description: Merges the current branch's PR, sees the post-merge runs through, and
  brings local main up to date. A PR number can be given as an argument.
argument-hint: "[pr-number]"
allowed-tools: Agent, AskUserQuestion
disable-model-invocation: true
---

```
/merge        → merger (merges the current branch's PR)
/merge 42     → merger (merges PR 42)
```

The actual work belongs to the `merger` agent. This skill is responsible only
for **argument parsing, dispatch, branching on the return value, and talking to
the user**. The procedures for checking CI state, merging, waiting for
post-merge, syncing main, and cleaning up branches all live in
`.claude/agents/merger.md`. **Do not copy them back here.**

`merger` returns only a count summary and the URLs of failed runs — no run
tables, no wait logs. **Do not go fetch the details it did not return, with
`gh run list` or otherwise** — keeping them out of main's context is that
agent's reason to exist.

## 0. Parsing arguments

A bare numeric token in `$ARGUMENTS` (e.g. `42`) is the PR number.

If there is none, **do not pass a PR number**. `merger` resolves it from the
current branch, so do not run `gh pr view` yourself.

## 1. Starting `merger`

Do not specify `model` (leave it to `merger`'s frontmatter).

**`/merge` is a command a human types, and typing it is the approval to merge**,
so pass the approved flag from the very first dispatch. `merger` skips the
approval judgment and goes straight to merging, so `NEEDS_APPROVAL` never comes
back on this path.

(That premise is upheld by `disable-model-invocation: true` in the frontmatter.
Remove it and Claude could start this skill on its own initiative and attach the
approved flag, so do not remove it.)

```
Agent(
  subagent_type: "merger",
  run_in_background: false,
  prompt: "PR number: <number>
Approved: yes (the user approved the merge by invoking /merge)

Skip the approval judgment and go from the merge through post-merge waiting, main sync, and branch cleanup.")
```

If the PR number is unknown, omit the `PR number:` line entirely.

**Do not set `run_in_background: true`.** Waiting for CI and waiting for
post-merge are carved into the foreground inside the agent.

## 2. Branching on the return value

| Terminal state                             | What to do                                                                                                                                                                   |
| ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MERGED`                                   | Report the merge commit SHA, whether this was a fresh merge or a recovery of an already-merged PR, and the post-merge summary, and finish                                     |
| `NOT_READY`                                | Report the observed `ACTION`, point the user at `/pr <number>` to resolve it, and finish. **Do not go fix the conflict or CI yourself** (`merger` has no fixing work either — that is `pr-runner`'s job) |
| `POST_MERGE_FAILED` / `POST_MERGE_TIMEOUT` | Report the failed run URLs / how long it waited and ask for direction. Add that **the merge itself completed and only the main sync and branch cleanup have not run**          |
| `FAILED`                                   | Report the reason, which step it stopped at, and whether the merge completed, then ask for direction. Only when the reason is `pr_not_found`, confirm the PR number with `AskUserQuestion` and redo step 1 with it |
