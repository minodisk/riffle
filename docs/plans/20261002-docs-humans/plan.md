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

# Move the human-facing docs into docs/humans/

## Purpose

`docs/` mixes two audiences: `docs/agents/` holds the guides for the coding
agents, while the four human-facing documents (`cameras.md`, `performance.md`,
`raw-formats.md`, `usage.md`) sit loose at the top level next to `plans/`.
Moving them into `docs/humans/` makes the layout symmetric (`agents/`,
`humans/`, `plans/`) so a reader or an agent can tell from the path who a
document is for, and gives future human-facing docs an obvious home.

## Steps

- [x] Step 1: Move the four docs to `docs/humans/` and update every live reference
  - Done when:
    - `docs/cameras.md`, `docs/performance.md`, `docs/raw-formats.md` and
      `docs/usage.md` live at `docs/humans/<name>.md`, moved with `git mv` so
      `git log --follow` keeps their history; `docs/` then contains only
      `agents/`, `humans/`, `plans/`.
    - `git grep -n -E 'docs/(cameras|performance|raw-formats|usage)\.md' -- ':!docs/plans/_archived' ':!docs/plans/review-history'`
      returns nothing, and
      `git grep -n -E '\]\(\./(cameras|performance|raw-formats|usage)\.md' -- 'docs/*.md' docs/agents`
      returns nothing (the moved files' links to each other now live under
      `docs/humans/` and stay `./<name>.md`).
    - Every relative link in the moved files resolves from `docs/humans/`:
      the `../README.md#compatibility` (cameras.md lines 5 and 13),
      `../README.md` (usage.md line 3) and `../CONTRIBUTING.md` (usage.md
      line 710) links become `../../README.md...` / `../../CONTRIBUTING.md`;
      the `./cameras.md` / `./usage.md` / `./raw-formats.md` cross-links are
      unchanged.
    - `README.md` and `README.ja.md` change in the same PR, with the same
      set of links repointed (11 and 10 lines respectively).
    - `mise run ci` passes (lychee with `--include-fragments` checks the
      `#installing`, `#keys`, `#mcp-companion` anchors into
      `docs/humans/usage.md` and the `#compatibility` anchor into README).
  - Implementation approach:
    - `git mv docs/cameras.md docs/humans/cameras.md` (and the other three);
      create `docs/humans/` with the first move.
    - Files to repoint `docs/<name>.md` -> `docs/humans/<name>.md`:
      `README.md` (lines 18, 42, 108, 126, 143, 182, 292, 327, 329, 331, 357),
      `README.ja.md` (lines 13, 33, 57, 58, 61, 68, 128 and the compatibility /
      links section), `CONTRIBUTING.md` line 20, `crates/core/src/scan.rs`
      line 320 (doc comment ``compare with `docs/performance.md` ``),
      `docs/agents/raw-metadata-parsing.md` line 63, `docs/agents/tauri-app.md`
      line 1037, and the 23 backticked / prose mentions in `todo.md` (they are
      not links, but the acceptance grep covers them, and a stale path in an
      open todo misleads the next agent). Use a mechanical replace of the
      four exact strings and review the diff; do not reword surrounding text.
    - Inside the moved files, change only the `../README.md` and
      `../CONTRIBUTING.md` targets listed above; leave the `./<name>.md`
      cross-links alone.
    - `CLAUDE.md`, `.claude/**`, `.github/**`, `release-please-config.json`
      and `package.json` hold no reference to the four files (verified by
      grep at planning time); re-run the grep before opening the PR rather
      than assuming.
    - Leave `docs/plans/_archived/**` and `docs/plans/review-history/**`
      untouched: they are historical records, their mentions of the old
      paths are backticked or prose (no live Markdown links), and lychee
      already excludes `review-history`, so nothing breaks. State this in
      the PR body.
    - Commit as `docs: move the human-facing docs into docs/humans/`
      (Conventional Commits; one PR).

## Trade-offs and risks

- Updating `todo.md` mentions: they are prose, not links, so skipping them
  would not break CI, but the acceptance grep would then still find old
  paths and future readers would be sent to a missing file. The plan updates
  them; the alternative (leave `todo.md` as a record) is only worth it if the
  caller prefers a smaller diff.
- `docs/plans/_archived/**` is intentionally left stale. If a future lychee
  upgrade or a change to the `--exclude-path` list starts checking a link
  there, fix that link individually at that time rather than rewriting the
  archive now.
- `git mv` plus edits to the moved files in one commit can lower git's
  rename similarity for `usage.md` (two one-line link edits in a ~700-line
  file; well above the 50% default threshold, so `--follow` should still
  work). If `git log --follow` loses the history, split into two commits in
  the same PR: the pure move, then the link edits.
- The directory name `humans/` is the user's choice, symmetric with
  `docs/agents/`.

## Progress

- (none yet)
