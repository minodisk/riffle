<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Run merger's step 6 commands as separate, verbatim Bash calls

## Purpose

The `merger` agent sometimes stops at step 6 with `FAILED` (reason:
`post_merge_sync_failed`) after the merge and post-merge runs already
succeeded. In the PR #657 session it ran, in one Bash call with
`dangerouslyDisableSandbox: true`,
`mise run git:main 2>&1 | tail -5; ./tools/git/delete_merged_branches.sh 2>&1 | tail -5`,
and Claude Code's auto mode classifier denied it as a "[Safety Bypass Flag]".

`.claude/settings.json` allows exactly `Bash(mise run git:main)` and
`Bash(./tools/git/delete_merged_branches.sh)` (plus `Bash(mise run *)`). A
piped, redirected or `;`-chained line matches none of them, so it falls to the
classifier, which nondeterministically denies the sandbox bypass. Measured
across past subagent transcripts: the bare `mise run git:main` with the flag
was allowed 137/137 times; all 4 denials were `| tail -N` or `;`-chained
variants (the chained form also passed 21 times).

`merger.md` step 6 already shows the two commands in separate code blocks but
never says they must be two separate Bash calls or that the text must be
verbatim. Stating that rule, with its reason, removes the failure mode without
touching any script or permission rule.

## Steps

- [x] Step 1: State the separate-call, verbatim rule in `merger.md` step 6
  - Done when:
    - `.claude/agents/merger.md` step 6 ("Sync main and clean up branches")
      says that `mise run git:main` and `./tools/git/delete_merged_branches.sh`
      run as **two separate Bash calls**, each **exactly as written in its code
      block**: no pipes, no redirects (`2>&1`), no `tail`, no `;` / `&&`
      chaining, no `cd` prefix
    - It explains why: anything else misses the allow rules in
      `.claude/settings.json`, goes to the auto mode classifier, which may deny
      the sandbox bypass as a Safety Bypass, and the merger then stops with
      `post_merge_sync_failed` after a merge that already succeeded
    - Only `.claude/agents/merger.md` changes; no script, settings or other
      skill file is touched
    - `mise run ci` passes
  - Implementation approach:
    - Edit only step 6 of `.claude/agents/merger.md`. Keep the existing
      structure (code block, bold rule paragraph, rationale) and the file's
      tone: bold imperative rule, then the measured reason, as the existing
      `**Run it by its relative path.**` and
      `**Always pass `dangerouslyDisableSandbox: true` ...**` paragraphs do
    - Place the rule once, as a short paragraph right under the step heading
      covering both commands. Do not duplicate the full rationale twice
    - Mention the measurement briefly (bare command allowed every time; the
      denials were all piped or chained variants; chaining is nondeterministic)
      so a future reader does not "simplify" it back into one call
    - The existing "limit it to this one command" wording about the sandbox
      bypass stays; the new rule is about command shape, not about the flag
    - English, Prettier-formatted Markdown (`mise run fmt` before committing;
      `mise run ci` to verify)

## Trade-offs and risks

- Scope of the rule. `.claude/skills/pr/SKILL.md` and
  `.claude/skills/develop/README.md` also mention `mise run git:main`, but
  neither chains the command and neither is executed by an unattended
  subagent, so they stay untouched.
- This is a prose fix; it cannot be unit-tested. The only verification is
  `mise run ci` and the next real merger run.

## Progress

- (none yet)
