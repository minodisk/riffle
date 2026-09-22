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

# Explicit Bash `timeout` for long-waiting agents

## Purpose

`merger` and `pr-runner` run `wait-pr-actionable.sh` (default 240 s of sleeps,
about 5 minutes real time) and `wait-post-merge-runs.sh --max-wait=240` in the
foreground without passing the Bash tool's `timeout` parameter. The agent and
script docs assume the Bash tool's foreground default is 6 minutes, but in this
harness it is 120 s: the harness moves the command to the background
("Command did not complete within its 120s timeout and was moved to the
background"), the agent improvises `sleep` / `until` polling (backgrounded
again at 300 s), eventually hands back `MERGED` / `ready`, and the background
shells linger, so the subagent shows as "running" for hours in the UI.

After this work every documented long-running command carries
`timeout: 600000` explicitly, the stale "6-minute default" comments say what
the default actually can be, and the long-waiting agents have a rule that they
never improvise polling when the harness backgrounds a command and stop their
own background tasks before handing back. The existing ban on
`run_in_background: true` stays: a subagent that waits on a background task
ends its turn and hands back an interim result, so the ban's premise still
holds.

## Steps

- [x] Step 1: Require `timeout: 600000` at every long-wait call site, fix the
      stale default-timeout comments, and add the no-improvised-polling /
      stop-own-background-tasks rule to the long-waiting agents
  - Done when:
    - Every place an agent or skill doc tells an agent to run
      `wait-pr-actionable.sh`, `wait-post-merge-runs.sh`, `mise run ci`,
      `commit-push.sh` / `commit-review-history.sh` (they run `mise run ci`
      inside), or any other command expected to exceed 2 minutes in the
      foreground states that the Bash tool's `timeout: 600000` (ms) must be
      passed explicitly. Known call sites (verify with
      `grep -rn "wait-pr-actionable\|wait-post-merge-runs\|mise run ci\|commit-push\|commit-review-history" .claude`):
      - `.claude/agents/merger.md` step 1 (`wait-pr-actionable.sh`, ~L59-92)
        and step 5 (`wait-post-merge-runs.sh --max-wait=240`, ~L158-202).
        Neither mentions `timeout` today.
      - `.claude/agents/pr-runner.md` §4 wait loop (~L145-250; the
        `<wait-pr-actionable command>` block and the "4-minute default timeout
        exists because ... 6 minutes by default and 10 at most" paragraph at
        ~L234-237) and the Claude execution contract (~L490-500, add the
        `timeout: 600000` requirement to the `wait-pr-actionable` and
        `commit-push` entries). `mise run ci` at ~L91-97 already says
        `600000`; only its "6-minute default" wording changes.
      - `mise run ci` paragraphs that already say `600000` but justify it with
        "the 6-minute default would cut it off": `implementer.md` ~L55-58,
        `pr-check-fixer.md` ~L62-65, `pr-conflict-resolver.md` ~L136-139,
        `pr-review-addresser.md` ~L71-74, `local-review-addresser.md`
        ~L80-83, `pr-runner.md` ~L94-97. Reword to "the default timeout can
        be as low as 120 s".
      - `local-review-runner.md` ~L91-100 and ~L166-169 already state
        `timeout: 600000` for `commit-review-history.sh`; confirm and leave
        unless wording refers to a 6-minute default.
      - Skill docs that run long commands in the main session: check
        `.claude/skills/{develop,pr,merge,release}/SKILL.md`,
        `.claude/skills/develop/references/pr-merge-lifecycle.md` and
        `.claude/skills/develop/README.md`. They delegate the waits to
        `merger` / `pr-runner`; `watch-local-review.sh` is deliberately
        `run_in_background: true` in the main session and is out of scope. If
        a direct long-running call is found, add the `timeout: 600000` note
        there too; otherwise no change.
    - The header comment of `.claude/skills/pr/scripts/wait-pr-actionable.sh`
      (~L15-23) no longer says "6 minutes by default and 10 at most" or
      "comfortably under the 6-minute default"; it says the Bash tool's
      default foreground timeout can be as low as 120 s, that callers must
      pass `timeout: 600000` explicitly, and that the 240 s default keeps real
      time well under 600 s. Rewrite the "To wait 30 minutes, start it with
      run_in_background: true" sentence so it does not contradict the agents'
      ban ("callers extend the wait by re-running and counting exit 2, not by
      backgrounding"). Defaults, `usage:` line and exit codes unchanged.
    - The header comment of
      `.claude/skills/merge/scripts/wait-post-merge-runs.sh` gains a note next
      to `MAX_WAIT` (~L17) that agent callers pass `--max-wait=240` and the
      Bash tool's `timeout: 600000`, because the default foreground timeout
      can be as low as 120 s. Defaults and exit codes unchanged.
    - `merger.md` step 1 and step 5, and `pr-runner.md` §4, contain a rule:
      if the harness nonetheless reports that a command was moved to the
      background, do not improvise `sleep` / `until` polling or any other
      waiting command; re-run the same script in the foreground with
      `timeout: 600000` (counters keep counting exit 2 as before); and before
      handing back any result, stop every background task of your own that is
      still running with `TaskStop` so nothing lingers after the hand-back.
    - The `tools:` frontmatter of `merger.md` and `pr-runner.md` includes
      `TaskStop`. The other agents (whose only long command is `mise run ci`)
      do not get the rule or `TaskStop`.
    - `mise run ci` passes.
    - `learnings.md` notes that the fix is verified only statically here: a
      real `/pr` or `/merge` run must be observed to confirm the `merger` /
      `pr-runner` subagents end with no lingering background tasks, and
      records the observed 120 s default that motivated the change.
  - Implementation approach:
    - Docs and script comments only; no code path or exit code changes. Match
      the surrounding style (bold imperative "Do not ..." sentences, the
      existing counter vocabulary).
    - Keep the existing "Do not switch the waiting to `run_in_background:
      true`" paragraphs; insert the `timeout: 600000` requirement and the new
      rule next to them. In `merger.md` ~L197-202 replace the "(6 minutes by
      default, 10 at most)" justification with the 120 s fact.
    - Both wait scripts at 240 s stay well under 600 s real time; keep `240`.
    - `lychee --offline --include-fragments` runs over `.claude/**/*.md`, so
      any new intra-repo link must resolve.

## Trade-offs and risks

- The `TaskStop` rule is limited to `merger` / `pr-runner` (observed
  offenders); the `mise run ci` agents already pass `timeout: 600000`.
- Verification is static only. Whether `TaskStop` reaches auto-backgrounded
  shells from a subagent can only be confirmed on a real run.

## Progress

(2026-09-22) Step 1 complete
