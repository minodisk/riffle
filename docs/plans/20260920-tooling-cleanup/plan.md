<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Close out the remaining Tooling items in todo.md

## Purpose

`todo.md`'s "Tooling / CI" section still lists five items not covered by the
in-flight `docs/plans/20260919-todo-cleanup/plan.md` (which owns the
`origin/HEAD` guard and the `git:main` fast-forward). Investigation shows two of
the five are already satisfied and need no code:

- **`.gitignore` missing `tmp/`**: `/tmp` has been in the tracked `.gitignore`
  since #122 (`git check-ignore tmp/x` matches). Closed as already done.
- **No `.claude/settings.json`**: the file has been tracked since #107 and
  carries exactly the allowlist the skills describe (`Bash(date *)`,
  `Bash(bash .claude/skills/*)`, `Bash(mise run *)`), and
  `.claude/settings.local.json` is ignored by the user's global gitignore. The
  skill text that says the allowlist exists is therefore true, and
  `settings-promoter`'s "no file" case cannot occur. Closed as already done;
  no skill text changes.

The other three get one small PR each: drop the `ai-coauthored` label (user
decision: the label is unnecessary), add a Markdown link checker to
`mise run lint` (and so to CI), and stop `tools/git/delete_merged_branches.sh`
from deleting or detaching the branch the next step is working on. Removing the
five closed `todo.md` headings happens in wrap-up, not in a step.

## Steps

- [ ] Step 1: Remove the `ai-coauthored` label from the PR tooling and from GitHub
  - Done when:
    - `.claude/skills/pr/scripts/create-pr.sh` no longer passes
      `--label ai-coauthored`, and its usage text no longer mentions the label.
    - `grep -rn ai-coauthored` over the repository (excluding `todo.md`, which
      wrap-up removes, and `.git`/`node_modules`/`target`) returns nothing.
    - The GitHub label is gone: `gh label delete ai-coauthored --yes` was run
      and `gh label list` no longer shows it.
    - `mise run ci` passes (shellcheck covers the script).
  - Implementation approach:
    - Only two lines in `create-pr.sh` reference the label (the usage heredoc
      line "Create a PR labeled ai-coauthored..." and the final `gh pr create`
      line). No other file in `.claude/`, `docs/`, `README.md` or
      `CONTRIBUTING.md` mentions it (measured with grep).
    - Delete the GitHub label as a one-off command during the step and record
      it in `learnings.md`; deleting a label strips it from past PRs, which the
      user has accepted.
    - This PR touches `.claude/skills/**`, so `check-merge-approval.sh` flags
      it; per the develop skill the read-only result goes into the step's
      progress report.

- [ ] Step 2: Add a Markdown link checker to `mise run lint`
  - Done when:
    - `mise.toml` declares the checker under `[tools]` with a pinned version and
      the `lint` task runs it, so both `mise run ci` and the CI `lint` job (which
      already runs `mise run lint` after `jdx/mise-action`) check Markdown
      links and in-file anchors (`#heading` fragments).
    - The check passes on the current tree, and a deliberately broken anchor in
      `README.md` makes it fail (verified locally, then reverted).
    - `CONTRIBUTING.md`'s "Checks" bullet for `mise run lint` names the link
      check alongside shellcheck/actionlint.
    - `mise run ci` passes.
  - Implementation approach:
    - Tool: `lychee` (in the mise registry as `aqua:lycheeverse/lychee`;
      cross-platform, no Node dependency, understands Markdown fragments with
      `--include-fragments`). Add it as `lychee = "<pinned version>"` next to
      `actionlint`/`shellcheck`.
    - Run it in offline mode so CI never fails on external-site flakiness:
      something like `lychee --offline --include-fragments --no-progress README.md CONTRIBUTING.md CLAUDE.md 'docs/**/*.md'`.
      Decide the file set during the step: start with the root Markdown files
      and `docs/agents/`; measure whether `docs/plans/**` and `.claude/**/*.md`
      pass as-is before including them (archived plans may hold stale relative
      links; excluding them is acceptable if so, noted in `learnings.md`).
    - Check that `--offline` still verifies relative file links and anchors;
      if lychee's offline mode skips fragments, drop `--offline` and instead
      exclude remote URLs with `--exclude` / `--scheme file`.
    - No `.github/workflows/ci.yml` change should be needed because the lint
      job runs `mise run lint`; confirm on the PR's CI run.

- [ ] Step 3: Make `delete_merged_branches.sh` safe against the next step's freshly cut branch
  - Done when:
    - `tools/git/delete_merged_branches.sh` no longer changes the current
      worktree's checkout (no `git checkout --detach`), so a branch checked out
      in any worktree, including the one the script runs in, is never deleted
      or detached out from under an implementer, and running the script twice
      in a row (the duplicate-`MERGED` case) is a no-op the second time.
    - Branches that are merged into `origin/<default>` are still deleted, and
      squash-merged branches still are too (behaviour verified in a throwaway
      clone under the scratchpad, never in this worktree: a merged branch, a
      squashed branch, a fresh commit-less branch that is checked out, and a
      fresh commit-less branch that is not checked out).
    - `.claude/agents/merger.md` (the sentence "The script detaches HEAD at
      `origin/<default>` before judging, so it does not conflict...") and the
      hazard text in `.claude/skills/develop/SKILL.md` (§ Merge, "Do not create
      the next branch until merger has returned MERGED") and
      `.claude/skills/develop/references/pr-merge-lifecycle.md` ("Its branch
      cleanup deletes every local branch already merged...") describe the new
      behaviour accurately.
    - `mise run ci` passes (shellcheck runs on `tools/git/*.sh`).
  - Implementation approach:
    - Root cause (measured from the script): it runs
      `git checkout -q --detach "$targetRef"` in the current worktree only so
      that `git branch -d`'s "merged into HEAD" test agrees with
      `--merged=$targetRef`. That detach is what removes the `worktreepath`
      protection from the branch the next step just cut, and a commit-less
      branch at `origin/main` is trivially `--merged`, so `-d` deletes it. The
      implementer's next commit then lands on the detached HEAD (exactly what
      the todo entry reports).
    - Fix: drop the detach and delete with `git branch -D` in the first loop.
      `--merged=$targetRef` has already established the branch is contained in
      `origin/<default>`, which is the same guarantee `-d` would re-check
      against HEAD, so `-D` loses no safety. The existing
      `worktreepath`/target-branch skip then protects the checked-out branch.
      The second (squash) loop already uses `-D` and needs no change.
    - Optional belt-and-braces (decide during the step): also skip branches
      whose tip equals `$targetRef` in both loops, so a commit-less branch is
      never removed either. Keep it only if it does not complicate the loop;
      note the decision in `learnings.md`.
    - Sequencing: `docs/plans/20260919-todo-cleanup/plan.md` Step 2 edits the
      same function (the `origin/HEAD` symbolic-ref guard). Do this step after
      that PR merges, or rebase onto it; do not fold its scope in here.
    - The `merger.md` "Always pass dangerouslyDisableSandbox" paragraph for the
      script stays: the `.git/config` section cleanup still needs it.
    - This PR touches `tools/git/**` and `.claude/**`, both approval patterns
      in `check-merge-approval.sh`; report the read-only result as usual.

## Trade-offs and risks

- **Items closed without code.** `tmp/` in `.gitignore` and the
  `.claude/settings.json` policy are already the state of the repository. Their
  `todo.md` headings are removed at wrap-up.
- **Label removal is destructive on GitHub.** `gh label delete` removes the
  label from every past PR that carried it; the user decided the label is
  unnecessary.
- **Offline link checking** never fails on outages but does not catch a dead
  external URL; switching to online later is a one-flag change.
- **Scope of Markdown files.** `docs/plans/**` and `.claude/**/*.md` may be
  excluded if they surface stale links.
- **`-D` instead of `-d`.** Relies on `--merged=$targetRef`, the same
  containment test evaluated against the remote tip.
- **Overlap with `20260919-todo-cleanup` Step 2.** Both edit
  `tools/git/delete_merged_branches.sh`; Step 3 lands after it.

## Progress

- (not started)
