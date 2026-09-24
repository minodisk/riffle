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

# Japanese README and camera-capability table

## Purpose

Riffle's README is English only. Add `README.ja.md`, a permanent Japanese
translation kept in the repository, and land the README changes the user has
already drafted and reviewed: a centered app icon at the top, a
"What the camera records" table under Compatibility (AF point, AF frame size,
face tracking and sub-second capture time per camera, and which features each
affects), and a sentence in the Bursts bullet that links to it. Both READMEs
get a language-switch row so the Japanese file can be found from the repository
page. Because `CLAUDE.md` requires everything in the repository to be English,
it also needs an explicit exception for `README.ja.md` and a rule that keeps the
two READMEs in sync.

## Steps

- [x] Step 1: Land the reviewed README drafts with language-switch links, add `README.ja.md`, and amend the `CLAUDE.md` language rule
  - Done when:
    - `README.md` equals the reviewed draft
      (`/tmp/claude-1000/-home-minodisk--herdr-worktrees-riffle-worktree-rapid-forest-c55e/3867b48f-7505-4c85-960d-2bc3ae49773f/scratchpad/readme-drafts/README.md`;
      `README.md.diff` there is its diff against `49f334c`) except for the
      added language-switch row: icon at the top, the
      "#### What the camera records" subsection under
      "RAW formats and cameras", and the Bursts sentence linking to it.
    - `README.ja.md` equals the reviewed draft
      (`.../scratchpad/readme-drafts/README.ja.md`) except for the added
      language-switch row: full translation with the same additions; UI menu
      names and Lightroom menu paths stay in English; links to English-only
      docs carry "（英語）"; Lightroom is listed before PhotoLab wherever both
      appear.
    - Both files have a language-switch row right under the centered icon and
      above `# Riffle`, centered like the icon. `README.md` shows
      `English | [日本語](./README.ja.md)` and `README.ja.md` shows
      `[English](./README.md) | 日本語` (the current language unlinked).
    - `CLAUDE.md` "## Language" states that `README.ja.md` is the one file
      whose body is Japanese, and that any change to `README.md` must update
      `README.ja.md` in the same PR (and vice versa).
    - `mise run ci` passes (in particular `lychee --include-fragments` over
      `*.md`, which now also checks `README.ja.md`).
  - Implementation approach:
    - The drafts are no longer in the worktree (they were stashed before the
      branch was cut). Copy both files from the scratchpad directory above into
      the repository root, verify with `diff -q`, then add only the
      language-switch row. Do not otherwise re-edit the drafts.
    - `CLAUDE.md`: append two sentences to the existing "## Language"
      paragraph rather than adding a new section, e.g.
      "The one exception is `README.ja.md`, the Japanese translation of
      `README.md`; its body is Japanese. Keep the two in sync: a PR that
      changes `README.md` updates `README.ja.md` in the same PR, and vice
      versa." No agent or skill file mentions README language rules today,
      so nothing else needs to change.
    - Do not touch `docs/agents/`; the sync rule stays in `CLAUDE.md` only.
    - Markdown is excluded from `vp fmt` (`vite.config.ts`), so no formatter
      will rewrite either README.
    - Commit message: `docs: add README.ja.md and the camera-capability table`.
    - This is the only step, so the `develop` skill runs it in single-PR
      mode (Phase S).

## Trade-offs and risks

- **Language-switch links.** The user chose to add them (2026-09-24), so the
  files diverge from the reviewed drafts by that one row in each.
- **Where the sync rule lives.** `CLAUDE.md` only (chosen) versus also in
  `.claude/agents/implementer.md` / `local-review-reviewer.md`. Every agent
  already reads `CLAUDE.md`, and one place is the minimal option.
- **Translation drift.** Nothing mechanical enforces that `README.ja.md`
  tracks `README.md`; the rule relies on review. A CI check was considered and
  rejected as over-engineering for a two-file rule.
- **Anchor fragility.** `README.ja.md` links to Japanese heading anchors
  (`#対応状況`, `#カメラが記録する情報と使える機能`). lychee resolves them
  today (verified, 0 errors); renaming a Japanese heading later must update
  the link or lint fails.
- **Table facts.** The "What the camera records" table encodes the user's
  measurement that SIGMA BF and SIGMA fp L record no `SubSecTimeOriginal`,
  and AF/sharpness behavior from `crates/core/src/arw.rs` and
  `crates/core/src/sharpness.rs`. If the implementer notices a mismatch with
  the code, record it in `learnings.md` and ask rather than silently editing
  the reviewed table.

## Progress

- (none yet)
