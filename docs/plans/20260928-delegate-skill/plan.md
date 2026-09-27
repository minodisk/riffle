<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Delegate skill: hand a task to a fresh Claude session in a new herdr worktree

## Purpose

Starting a second development task today means leaving the current session,
creating a herdr worktree by hand, starting Claude in it, and typing
`/develop ...` again. The user verified that whole sequence by hand with the
`herdr` CLI. A `delegate` skill turns it into one command run from the current
session, so a task noticed mid-conversation (a bug seen on real hardware, a
follow-up outside the current plan) can be handed to a fresh, isolated session
without interrupting the current one.

## Steps

- [x] Step 1: Add `.claude/skills/delegate/SKILL.md`
  - Done when:
    - `.claude/skills/delegate/SKILL.md` exists with frontmatter `name: delegate`,
      a `description`, `argument-hint: "[task description]"`,
      `allowed-tools: Bash`, and `disable-model-invocation: true`
    - The body is written in English in the style of
      `.claude/skills/merge-settings/SKILL.md` (a short usage block like the
      other skills, then `## 1.` … `## 5.` numbered sections, one bash block
      each, "Report the result of each step to the user." at the end) and
      describes exactly these five steps:
      1. Verify the caller runs inside herdr: `test "${HERDR_ENV:-}" = 1`. If
         it fails, tell the user the session is not running inside Herdr and
         stop.
      2. Find the main checkout with `herdr worktree list` and read
         `.result.source.source_checkout_path`. Run
         `git -C <main checkout> fetch origin main`, then
         `herdr worktree create --cwd <main checkout> --base origin/main --no-focus`.
         From its JSON read `.result.root_pane.pane_id` and
         `.result.workspace.worktree.checkout_path` (the worktree directory,
         e.g. `worktree-brave-forest-9d98`); also keep the workspace ID for
         the report.
      3. `herdr agent start <name> --kind claude --pane <pane_id> --timeout 120000`.
         `<name>` must match `[a-z][a-z0-9_-]{0,31}`, be unique among live
         agents (check `herdr agent list`), and be a short name describing
         the task (e.g. `burst-badge`).
      4. `herdr agent prompt <name> "/develop <task description>"` **without**
         `--wait` (`/develop` runs for a long time). The description must stand
         alone for a session with no shared context: symptom, cause, related
         files, and evidence (e.g. "the user confirmed it on real hardware").
      5. Report to the user the worktree name, the workspace ID, and the agent
         name.
    - The body carries these notes: `herdr --skill` prints herdr's official
      skill text, and it plus `herdr <group> --help` are the source of truth
      for command syntax (re-check them if a command fails); never run a
      mutating herdr command with arguments omitted to "see the help"
      (`herdr worktree create` runs with defaults even with no arguments; use
      `herdr worktree create --help`); JSON field names come from the actual
      response, not from memory; the new session picks its own branch through
      `/develop` (this skill does not pass `--branch`); run the commands with
      the Bash tool.
    - No doc list is updated: `CLAUDE.md`, `README.md`, `README.ja.md`,
      `docs/**`, and `.claude/agents/*.md` do not enumerate skills, so there
      is nothing to extend (do not add a list).
    - `.claude/settings.json` is unchanged (the user chose to keep herdr
      commands prompting).
    - `mise run ci` passes (lychee checks `.claude/**/*.md` offline with
      fragments, so any relative link in the skill must resolve; the
      formatter ignores `.claude/**` and `**/*.md`).
  - Implementation approach:
    - Write the file with the Write tool (`.claude/` is on the sandbox Bash
      write deny list, as `merge-settings/SKILL.md` notes).
    - Frontmatter and layout follow the existing skills: `merge-settings`
      for the numbered bash-step shape, `merge` / `release` for the usage
      block and `disable-model-invocation: true`.
    - Commands and flags were verified against herdr 0.9.1 during planning:
      `herdr worktree create [--cwd --base --branch --path --label --focus|--no-focus]`,
      `herdr agent start <NAME> --kind <KIND> --pane <ID> [--timeout <MS>]`
      (default 30000, max 300000), `herdr agent prompt <TARGET> <TEXT>
      [--wait] [--timeout]`, `herdr worktree list` →
      `.result.source.source_checkout_path`. Re-check with `--help` when
      writing; do not run `herdr worktree create` or `herdr agent start`
      while implementing.
    - Commit: `feat(skills): add the delegate skill that hands a task to a new herdr worktree`

## Trade-offs and risks

1. **`.claude/settings.json` permissions for `herdr`.** Decided: add nothing;
   every delegation prompts for the mutating commands.
2. **Skill name.** Decided: `delegate` (a verb like `develop` / `merge` /
   `release`).
3. **`disable-model-invocation: true`.** Decided: set, because creating a
   worktree and starting an agent is a side effect the model should not
   trigger from a loose mention.
4. **Fetching in the main checkout.** `git -C <main> fetch origin main`, as in
   the user's verified procedure; it updates remote refs only, so
   `--base origin/main` is current.
5. **Windows.** The `test "${HERDR_ENV:-}" = 1` check assumes Bash (the Bash
   tool is Git Bash here); the skill says to run these in the Bash tool.
6. **Agent start timeout.** 120000 ms was what the user verified; the default
   30000 ms may be too short for Claude's startup in a fresh worktree.

## Progress

- (none yet)
